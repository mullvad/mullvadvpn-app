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

    deinit {
        updateListenerTask?.cancel()
    }

    /// Start listening for transaction updates.
    func start() async {
        logger.debug("Starting StoreKit 2 transaction listener.")

        legacyStorePaymentManager.start()

        _ = try? await processOutstandingTransactions()

        updateListenerTask?.cancel()
        updateListenerTask = Task { [weak self] in
            guard let self else { return }

            // If the purchase was made out-of-band, we need not upload the receipt.
            for await verification in Transaction.updates {
                guard shouldProcessPayment(verification: verification) else {
                    continue
                }

                do {
                    try await verification.payloadValue.finish()
                } catch {
                    continue
                }

                await updateAccountData()
                addToProcessedTransactions(verification)
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
        try await Product.products(for: ProductId.allCases.map { $0.rawValue })
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
            didFailVerification(transaction: transaction, error: verificationFailure)
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

        for await verification in Transaction.unfinished {
            guard shouldProcessPayment(verification: verification) else {
                continue
            }

            try await uploadReceipt(verification: verification)

            let payload = try verification.payloadValue
            await payload.finish()

            addToProcessedTransactions(verification)

            let isStoreKit2Transaction = ProductId.allCases
                .map { $0.rawValue }
                .contains(payload.productID)

            timeAdded +=
                isStoreKit2Transaction
                ? timeFromProduct(id: payload.productID)
                : legacyStorePaymentManager.timeFromProduct(id: payload.productID)
        }

        await updateAccountData()

        return if timeAdded > 0 {
            .timeAdded(timeAdded)
        } else {
            .noTimeAdded
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
        let isStoreKit2Transaction = try ProductId.allCases
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

    // Returns time added, in seconds.
    private func timeFromProduct(id: String) -> TimeInterval {
        let product = ProductId(rawValue: id)

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
    private func didFailUploadingReceipt(error: Error) {
        notifyObservers(of: .failed(.receiptUpload(error)))
    }

    /// Handle failure to verify the payment transaction.
    ///
    /// - Parameter error: error thrown by the API client
    private func didFailVerification(transaction: Transaction, error: VerificationResult<Transaction>.VerificationError)
    {
        Task {
            await transaction.finish()
        }

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
            failure = .unknown(error)
        }

        notifyObservers(of: .failed(failure))
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

    func restorePurchases(
        for accountNumber: String,
        completionHandler: @escaping @Sendable (Result<REST.CreateApplePaymentResponse, Error>) -> Void
    ) -> Cancellable {
        legacyStorePaymentManager.restorePurchases(for: accountNumber, completionHandler: completionHandler)
    }
}
