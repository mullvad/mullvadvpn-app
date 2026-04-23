//
//  StorePaymentManager.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-10-29.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadTypes
import StoreKit

/// Manager responsible for handling App Store payments and passing StoreKit receipts to the backend.
///
/// - Warning: only interact with this object on the main queue.
final actor StorePaymentManager: @unchecked Sendable {
    private let logger = Logger(label: "StorePaymentManager")
    private var observerList = ObserverList<StorePaymentObserver>()
    private let interactor: StorePaymentManagerInteractor
    private var processedTransactionIds: Set<UInt64> = []
    private var updateListenerTask: Task<Void, Never>?

    /// Designated initializer
    ///
    /// - Parameters:
    ///   - interactor: interactor for communicating with API etc.
    init(interactor: StorePaymentManagerInteractor) {
        self.interactor = interactor
    }

    /// Start listening for transaction updates.
    func start() async {
        logger.debug("Starting StoreKit transaction listener")

        #if !DEBUG
            // Always clean up non-production transactions immediately. Reason for this is that if there
            // are any old unfinished sandbox transactions that has spilled over from TestFlight, they
            // will clog up the pipeline since they can never be finished or removed in production.
            await Self.finishOutstandingSandboxAndOldAPITransactions()
        #endif

        _ = try? await processOutstandingTransactions()

        updateListenerTask?.cancel()
        updateListenerTask = Task { [weak self] in
            guard let self else { return }

            // If the purchase was made out-of-band, we need not upload the receipt.
            for await verification in Transaction.updates {
                guard await shouldProcessPayment(verification: verification) else {
                    continue
                }

                await updateAccountData()
            }
        }
    }

    // MARK: Notifications

    func addPaymentObserver(_ observer: StorePaymentObserver) {
        observerList.append(observer)
    }

    // MARK: - Products and payments

    func products() async throws -> [Product] {
        try await Product.products(for: StoreSubscription.allCases.map { $0.rawValue })
    }

    func purchase(product: Product) async {
        logger.debug("Purchasing product: \(product.id)")

        let token: UUID
        do {
            token = try await self.getPaymentToken()
        } catch {
            didFailFetchingToken(error: error)
            return
        }

        let result: Product.PurchaseResult
        do {
            result = try await product.purchase(
                options: [.appAccountToken(token)]
            )
        } catch {
            didFailPurchase(error: error)
            return
        }

        switch result {
        case let .success(.verified(transaction)):
            await purchaseWasSuccessful(transaction: transaction)
        case let .success(.unverified(transaction, verificationFailure)):
            await didFailVerification(transaction: transaction, error: verificationFailure)
        case .userCancelled:
            userDidCancel()
        case .pending:
            didSuspendPurchase()
        @unknown default:
            fatalError("Unhandled purchase result \(result)")
        }
    }

    func processOutstandingTransactions() async throws -> StorePaymentOutcome {
        var timeAdded: TimeInterval = 0
        var failedOneOrMoreTransactions = false

        logger.debug("Processing outstanding transactions")

        for await verification in Transaction.unfinished {
            guard shouldProcessPayment(verification: verification) else {
                continue
            }

            do {
                try await uploadReceipt(verification: verification)
            } catch {
                failedOneOrMoreTransactions = true
                continue
            }

            let payload = try verification.payloadValue
            await payload.finish()

            addToProcessedTransactions(verification)
            timeAdded += timeFromProduct(id: payload.productID)
        }

        await updateAccountData()

        if failedOneOrMoreTransactions {
            throw StorePaymentError.receiptUpload
        }

        return if timeAdded > 0 {
            .timeAdded(timeAdded)
        } else {
            .noTimeAdded
        }
    }

    static func finishOutstandingSandboxAndOldAPITransactions() async {
        let logger = Logger(label: "StorePaymentManager")

        logger.debug("Finishing outstanding sandbox and old transactions")

        for await verification in Transaction.unfinished {
            guard let payload = try? verification.payloadValue else {
                logger.debug("Verification is missing a valid payload")
                continue
            }

            logger.debug("Unfinished transaction environment is \(payload.environment)")

            let isStagingEnvironment = payload.environment != .production
            let isOldAPI = !StoreSubscription.allCases
                .map { $0.rawValue }
                .contains(payload.productID)

            if isStagingEnvironment || isOldAPI {
                logger.debug(
                    "Finishing transaction. isStagingEnvironment: \(isStagingEnvironment), isOldAPI: \(isOldAPI)"
                )
                await payload.finish()
            } else {
                logger.debug(
                    "Skipping transaction. isStagingEnvironment: \(isStagingEnvironment), isOldAPI: \(isOldAPI)"
                )
            }
        }
    }

    // MARK: - Private methods

    private func getPaymentToken() async throws -> UUID {
        let result = await interactor.initPayment()

        switch result {
        case .success(let token): return token
        case .failure(let error): throw error
        }
    }

    private func uploadReceipt(verification: VerificationResult<Transaction>) async throws {
        let payload = try verification.payloadValue

        let logMessage: String =
            "Uploading receipt. "
            + "Product ID: \(payload.productID), "
            + "Environment: \(payload.environment), "
            + "Purchase date: \(payload.purchaseDate.safeLogFormatted), "
            + "Revocation date: \(payload.revocationDate?.safeLogFormatted ?? "none")"
        logger.debug(.init(stringLiteral: logMessage))

        let result = await interactor.checkPayment(jwsRepresentation: verification.jwsRepresentation)

        switch result {
        case .success(): return
        case .failure(let error): throw error
        }
    }

    private func purchaseWasSuccessful(transaction: Transaction) async {
        let verification = VerificationResult<Transaction>.verified(transaction)

        do {
            try await uploadReceipt(verification: verification)
            await updateAccountData()

            try await verification.payloadValue.finish()

            addToProcessedTransactions(verification)
            didPurchaseMoreTime(outcome: .timeAdded(timeFromProduct(id: transaction.productID)))
        } catch {
            didFailUploadingReceipt()
        }
    }

    private func updateAccountData() async {
        guard let accountNumber = await interactor.accountNumber else {
            return
        }

        logger.debug("Updating account data")

        let result = await interactor.getAccountData(accountNumber: accountNumber)

        switch result {
        case let .success(accountData):
            logger.info("Successfully updated account data. New expiry: \(accountData.expiry.logFormatted)")
            await interactor.updateAccountData(for: accountData)

        case let .failure(error):
            if !error.isOperationCancellationError {
                logger.error(error: error, message: "Failed to update account data.")
            }
        }
    }

    private func transactionHasBeenProcessed(_ verificationResult: VerificationResult<Transaction>) -> Bool {
        guard let transactionId = try? verificationResult.payloadValue.id else {
            return true
        }

        let hasAlreadyBeenProcessed = processedTransactionIds.contains(transactionId)
        if hasAlreadyBeenProcessed {
            logger.debug("Verification has already been processed")
        }

        return hasAlreadyBeenProcessed
    }

    private func addToProcessedTransactions(_ verificationResult: VerificationResult<Transaction>) {
        guard let transactionId = try? verificationResult.payloadValue.id else {
            return
        }

        logger.debug("Adding to processed transactions")

        _ = processedTransactionIds.insert(transactionId)
    }

    // Returns time added, in seconds.
    private func timeFromProduct(id: String) -> TimeInterval {
        let product = StoreSubscription(rawValue: id)

        return switch product {
        case .thirtyDays: Duration.days(30).timeInterval
        case .ninetyDays: Duration.days(90).timeInterval
        case .none: 0
        }
    }

    private func shouldProcessPayment(verification: VerificationResult<Transaction>) -> Bool {
        guard case VerificationResult<Transaction>.verified = verification else {
            logger.debug("Verification was not .verified, instead was: \(verification)")
            return false
        }

        if let revocationDate = try? verification.payloadValue.revocationDate {
            logger.debug("Verification was revoked at: \(revocationDate)")
            return false
        }

        return !transactionHasBeenProcessed(verification)
    }

    // MARK: Notifications

    /// Purchase was successful.
    private func didPurchaseMoreTime(outcome: StorePaymentOutcome) {
        logger.debug("Purchase successful")
        notifyObservers(of: .successfulPayment(outcome))
    }

    /// User cancelled purchase before it was completed.
    private func userDidCancel() {
        logger.debug("User cancelled purchase")
        notifyObservers(of: .userCancelled)
    }

    /// Purchase is still pending, transaction may be delivered asynchronously.
    private func didSuspendPurchase() {
        logger.debug("Did suspend purchase")
        notifyObservers(of: .pending)
    }

    /// Handle failure to fetch a payment token
    ///
    /// - Parameter error: error thrown by the API client
    private func didFailFetchingToken(error: Error) {
        logger.debug("Did fail fetching token, with error: \(error)")
        notifyObservers(of: .failed(.getPaymentToken(error)))
    }

    /// Handle failure to upload a payment receipt to the API. This transaction should be uploaded again.
    ///
    /// - Parameter error: error thrown by the API client
    private func didFailUploadingReceipt() {
        logger.debug("Did fail uploading receipt")
        notifyObservers(of: .failed(.receiptUpload))
    }

    /// Handle failure to verify the payment transaction.
    ///
    /// - Parameter error: error thrown by the API client
    private func didFailVerification(
        transaction: Transaction,
        error: VerificationResult<Transaction>.VerificationError
    ) async {
        await transaction.finish()

        logger.debug("Did fail verification, with error: \(error)")
        notifyObservers(of: .failed(.verification(error)))
    }

    /// Handle an error thrown from the Product.purchase call
    ///
    /// - Parameter error: the error that was thrown by the Product.purchase call
    private func didFailPurchase(error: Error) {
        let failure: StorePaymentError
        switch error {
        case let storeKitError as StoreKitError:
            failure = .storeKitError(storeKitError)

        case let purchaseError as Product.PurchaseError:
            failure = .purchaseError(purchaseError)

        default:
            failure = .unknown
        }

        logger.debug("Did fail purchase, with error: \(error)")
        notifyObservers(of: .failed(failure))
    }

    private func notifyObservers(of storeKitEvent: StorePaymentEvent) {
        observerList.notify { observer in
            Task { @MainActor in
                observer.storePaymentManager(didReceiveEvent: storeKitEvent)
            }
        }
    }
}
