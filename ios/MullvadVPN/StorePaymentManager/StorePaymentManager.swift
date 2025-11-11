//
//  StorePaymentManager.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-10-29.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadREST
import MullvadTypes
@preconcurrency import StoreKit

/// Manager responsible for handling App Store payments and passing StoreKit receipts to the backend.
///
/// - Warning: only interact with this object on the main queue.
final actor StorePaymentManager: @unchecked Sendable {
    private let logger = Logger(label: "StorePaymentManager")
    private var observerList = ObserverList<StorePaymentObserver>()
    private let interactor: StorePaymentManagerInteractor
    private var processedTransactionIds: Set<UInt64> = []
    private var updateListenerTask: Task<Void, Never>?

    // Legacy payment manager, kept around until Store Kit 2 is fully migrated and tested.
    private let legacyStorePaymentManager: LegacyStorePaymentManager

    /// Designated initializer
    ///
    /// - Parameters:
    ///   - backgroundTaskProvider: the background task provider.
    ///   - interactor: interactor for communicating with API etc.
    init(backgroundTaskProvider: BackgroundTaskProviding, interactor: StorePaymentManagerInteractor) {
        self.interactor = interactor

        legacyStorePaymentManager = LegacyStorePaymentManager(
            backgroundTaskProvider: backgroundTaskProvider,
            queue: .default(),
            transactionLog: .default,
            interactor: interactor
        )
    }

    /// Start listening for transaction updates.
    func start() async {
        logger.debug("Starting StoreKit 2 transaction listener.")

        #if !DEBUG
            // Always clean up non-production transactions immediately. Reason for this is that if there
            // are any old unfinished sandbox transactions that has spilled over from TestFlight, they
            // will clog up the pipeline since they can never be finished or removed in production.
            await finishOutstandingSandboxTransactions()
        #endif

        // Disabled so as not to have a parallell listener for SK 1 transactions running at the
        // same time as SK 2 listener. Enable when enabling SK 1 payment flow.
        // legacyStorePaymentManager.start()

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
        legacyStorePaymentManager.addPaymentObserver(observer)
    }

    // MARK: - Products and payments

    func products() async throws -> [Product] {
        try await Product.products(for: StoreSubscription.allCases.map { $0.rawValue })
    }

    func purchase(product: Product) async {
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

            let isStoreKit2Transaction = StoreSubscription.allCases
                .map { $0.rawValue }
                .contains(payload.productID)

            timeAdded +=
                isStoreKit2Transaction
                ? timeFromProduct(id: payload.productID)
                : legacyStorePaymentManager.timeFromProduct(id: payload.productID)
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

    static func finishOutstandingSandboxTransactions() async {
        for await verification in Transaction.unfinished {
            guard let payload = try? verification.payloadValue else {
                continue
            }

            if payload.environment != .production {
                await payload.finish()
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
        let isStoreKit2Transaction = try StoreSubscription.allCases
            .map { $0.rawValue }
            .contains(verification.payloadValue.productID)

        let result: Result<Void, Error>
        if isStoreKit2Transaction {
            result = await interactor.checkPayment(jwsRepresentation: verification.jwsRepresentation)
        } else {
            result = await interactor.legacySendReceipt()
        }

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

    private func finishOutstandingSandboxTransactions() async {
        for await verification in Transaction.unfinished {
            guard let payload = try? verification.payloadValue else {
                continue
            }

            logger.debug("Unfinished transaction environment is '\(payload.environment)'")

            if payload.environment != .production {
                logger.debug("Finishing transaction with environment '\(payload.environment)'")
                await payload.finish()
            }
        }
    }

    private func transactionHasBeenProcessed(_ verificationResult: VerificationResult<Transaction>) -> Bool {
        guard let transactionId = try? verificationResult.payloadValue.id else {
            return true
        }

        return processedTransactionIds.contains(transactionId)
    }

    private func addToProcessedTransactions(_ verificationResult: VerificationResult<Transaction>) {
        guard let transactionId = try? verificationResult.payloadValue.id else {
            return
        }

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
            return false
        }

        let revocationDate = try? verification.payloadValue.revocationDate
        return (revocationDate == nil) && !transactionHasBeenProcessed(verification)
    }

    // MARK: Notifications

    /// Purchase was successful.
    private func didPurchaseMoreTime(outcome: StorePaymentOutcome) {
        notifyObservers(of: .successfulPayment(outcome))
    }

    /// User cancelled purchase before it was completed.
    private func userDidCancel() {
        notifyObservers(of: .userCancelled)
    }

    /// Purchase is still pending, transaction may be delivered asynchronously.
    private func didSuspendPurchase() {
        notifyObservers(of: .pending)
    }

    /// Handle failure to fetch a payment token
    ///
    /// - Parameter error: error thrown by the API client
    private func didFailFetchingToken(error: Error) {
        notifyObservers(of: .failed(.getPaymentToken(error)))
    }

    /// Handle failure to upload a payment receipt to the API. This transaction should be uploaded again.
    ///
    /// - Parameter error: error thrown by the API client
    private func didFailUploadingReceipt() {
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
            logger.error("Caught unknown error during purchase call: \(error)")
            failure = .unknown
        }

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

// Proxy functions for legacy payment
extension StorePaymentManager {
    nonisolated func requestProducts(
        with productIdentifiers: Set<LegacyStoreSubscription>,
        completionHandler: @escaping @Sendable (Result<SKProductsResponse, Error>) -> Void
    ) -> Cancellable {
        legacyStorePaymentManager.requestProducts(with: productIdentifiers, completionHandler: completionHandler)
    }

    nonisolated func addPayment(_ payment: SKPayment, for accountNumber: String) async {
        await legacyStorePaymentManager.addPayment(payment, for: accountNumber)
    }

    nonisolated func restorePurchases(
        for accountNumber: String,
        completionHandler: @escaping @Sendable (Result<REST.CreateApplePaymentResponse, Error>) -> Void
    ) async -> Cancellable {
        await legacyStorePaymentManager.restorePurchases(for: accountNumber, completionHandler: completionHandler)
    }
}
