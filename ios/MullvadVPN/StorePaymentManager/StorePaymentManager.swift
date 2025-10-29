//
//  StorePaymentManager.swift
//  MullvadVPN
//
//  Created by pronebird on 10/03/2020.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadREST
import MullvadTypes
@preconcurrency import StoreKit

/// Manager responsible for handling AppStore payments and passing StoreKit receipts to the backend.
///
/// - Warning: only interact with this object on the main queue.
final class StorePaymentManager: @unchecked Sendable {
    private enum ProductId: String, CaseIterable {
        case thirtyDays = "net.mullvad.MullvadVPN.subscription.storekit2.30days"
        case ninetyDays = "net.mullvad.MullvadVPN.subscription.storekit2.90days"
    }

    private let logger = Logger(label: "StorePaymentManager")
    private var observerList = ObserverList<StorePaymentObserver>()
    private let interactor: StorePaymentManagerInteractor
    private let processedTransactionIdslock = NSLock()
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
    func start() {
        logger.debug("Starting StoreKit 2 transaction listener.")

        updateListenerTask = Task.detached { [weak self] in
            guard let self else { return }

            _ = try? await processUnfinishedTransactions()

            // If the purchase was made out-of-band, we need not upload the receipt.
            for await transaction in Transaction.updates {
                if case let .verified(purchase) = transaction {
                    if purchase.revocationDate != nil,
                        !transactionHasBeenProcessed(transaction)
                    {
                        await updateAccountData()
                        addToProcessedTransactions(transaction)
                    }
                }
            }
        }
    }

    // MARK: - Products and payments

    func products() async throws -> [Product] {
        try await Product.products(for: ProductId.allCases.map { $0.rawValue })
    }

    func purchase(product: Product, for accountNumber: String) async {
        let token: UUID
        do {
            token = try await self.getPaymentToken(for: accountNumber)
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
            await purchaseWasSuccessful(accountNumber: accountNumber, transaction: transaction)
        case let .success(.unverified(_, verificationFailure)):
            didFailVerification(error: verificationFailure)
        case .userCancelled:
            userDidCancel()
        case .pending:
            didSuspendPurchase()
        @unknown default:
            fatalError("Unhandled purchase result \(result)")
        }
    }

    func processUnfinishedTransactions() async throws -> StorePaymentOutcome {
        guard let accountNumber = interactor.accountNumber else {
            logger.error("No account number available for transaction.")
            return .noTimeAdded
        }

        var timeAdded: TimeInterval = 0

        // Attempt processing unfinished transactions.
        for await verification in Transaction.unfinished {
            guard try verification.payloadValue.revocationDate != nil,
                !transactionHasBeenProcessed(verification)
            else {
                continue
            }

            guard case VerificationResult<Transaction>.verified = verification else {
                logger.error("Failed to verify transaction.")
                continue
            }

            // Upload transaction to API
            try await uploadReceipt(accountNumber: accountNumber, jwsRepresentation: verification.jwsRepresentation)

            addToProcessedTransactions(verification)
            timeAdded += try timeFromProduct(id: verification.payloadValue.productID)
        }

        let outcome: StorePaymentOutcome
        if timeAdded > 0 {
            // Update account data if transactions were processed.
            await updateAccountData()
            outcome = .timeAdded(timeAdded)
        } else {
            outcome = .noTimeAdded
        }

        return outcome
    }

    // MARK: - Private methods

    private func getPaymentToken(for accountNumber: String) async throws -> UUID {
        let result = await interactor.initPayment(accountNumber: accountNumber)

        switch result {
        case .success(let token): return token
        case .failure(let error): throw error
        }
    }

    private func uploadReceipt(accountNumber: String, jwsRepresentation: String) async throws {
        let result = await interactor.checkPayment(accountNumber: accountNumber, jwsRepresentation: jwsRepresentation)

        switch result {
        case .success(): return
        case .failure(let error): throw error
        }
    }

    private func purchaseWasSuccessful(accountNumber: String, transaction: Transaction) async {
        let verification = VerificationResult<Transaction>.verified(transaction)

        do {
            try await uploadReceipt(accountNumber: accountNumber, jwsRepresentation: verification.jwsRepresentation)
            await updateAccountData()

            try await verification.payloadValue.finish()

            addToProcessedTransactions(verification)
            didPurchaseMoreTime(outcome: .timeAdded(timeFromProduct(id: transaction.productID)))
        } catch {
            didFailUploadingReceipt(error: error)
        }
    }

    private func updateAccountData() async {
        guard let accountNumber = interactor.accountNumber else {
            return
        }

        let result = await interactor.getAccountData(accountNumber: accountNumber)

        switch result {
        case let .success(accountData):
            logger.info("Successfully updated account data. New expiry: \(accountData.expiry.logFormatted)")

            // Notify delegate about successful account update.
            interactor.updateAccountData(for: accountData)

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

        return processedTransactionIdslock.withLock {
            processedTransactionIds.contains(transactionId)
        }
    }

    private func addToProcessedTransactions(_ verificationResult: VerificationResult<Transaction>) {
        guard let transactionId = try? verificationResult.payloadValue.id else {
            return
        }

        processedTransactionIdslock.withLock {
            _ = processedTransactionIds.insert(transactionId)
        }
    }

    private func timeFromProduct(id: String) -> TimeInterval {
        let product = ProductId(rawValue: id)

        return switch product {
        case .thirtyDays: 30
        case .ninetyDays: 90
        case .none: 0
        }
    }

    // MARK: Notifications

    /// Purchase was successful.
    private func didPurchaseMoreTime(outcome: StorePaymentOutcome) {
        notifyObservers(of: StorePaymentEvent.successfulPayment(outcome))
    }

    /// User cancelled purchase before it was completed.
    private func userDidCancel() {
        notifyObservers(of: StorePaymentEvent.userCancelled)
    }

    /// Purchase is still pending, transaction may be delivered asynchronously.
    private func didSuspendPurchase() {
        notifyObservers(of: StorePaymentEvent.pending)
    }

    /// Handle failure to fetch a payment token
    ///
    /// - Parameter error: error thrown by the API client
    private func didFailFetchingToken(error: Error) {
        notifyObservers(of: StorePaymentEvent.failed(.getPaymentToken(error)))
    }

    /// Handle failure to upload a payment receipt to the API. This transaction should be uploaded again.
    ///
    /// - Parameter error: error thrown by the API client
    private func didFailUploadingReceipt(error: Error) {
        notifyObservers(of: StorePaymentEvent.failed(.receiptUpload(error)))
    }

    /// Handle failure to verify the payment transaction.
    ///
    /// - Parameter error: error thrown by the API client
    private func didFailVerification(error: VerificationResult<Transaction>.VerificationError) {
        notifyObservers(of: StorePaymentEvent.failed(.verification(error)))
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
            failure = .unknown(error)
        }

        notifyObservers(of: StorePaymentEvent.failed(failure))
    }

    private func notifyObservers(of storeKitEvent: StorePaymentEvent) {
        observerList.notify { observer in
            observer.storePaymentManager(didReceiveEvent: storeKitEvent)
        }
    }
}

// Proxy functions for legacy payment
extension StorePaymentManager {
    func requestProducts(
        with productIdentifiers: Set<StoreSubscription>,
        completionHandler: @escaping @Sendable (Result<SKProductsResponse, Error>) -> Void
    ) -> Cancellable {
        legacyStorePaymentManager.requestProducts(with: productIdentifiers, completionHandler: completionHandler)
    }

    func addPayment(_ payment: SKPayment, for accountNumber: String) {
        legacyStorePaymentManager.addPayment(payment, for: accountNumber)
    }

    func addPaymentObserver(_ observer: StorePaymentObserver) {
        legacyStorePaymentManager.addPaymentObserver(observer)
    }

    func restorePurchases(
        for accountNumber: String,
        completionHandler: @escaping @Sendable (Result<REST.CreateApplePaymentResponse, Error>) -> Void
    ) -> Cancellable {
        legacyStorePaymentManager.restorePurchases(for: accountNumber, completionHandler: completionHandler)
    }
}
