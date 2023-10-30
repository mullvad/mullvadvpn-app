//
//  StorePaymentManager.swift
//  MullvadVPN
//
//  Created by pronebird on 10/03/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadREST
import MullvadTypes
import Operations
import StoreKit
import UIKit

/// Manager responsible for handling AppStore payments and passing StoreKit receipts to the backend.
///
/// - Warning: only interact with this object on the main queue.
final class StorePaymentManager: NSObject, SKPaymentTransactionObserver {
    private enum OperationCategory {
        static let sendStoreReceipt = "StorePaymentManager.sendStoreReceipt"
        static let productsRequest = "StorePaymentManager.productsRequest"
    }

    private let logger = Logger(label: "StorePaymentManager")

    private let operationQueue: OperationQueue = {
        let queue = AsyncOperationQueue()
        queue.name = "StorePaymentManagerQueue"
        return queue
    }()

    private let backgroundTaskProvider: BackgroundTaskProvider
    private let paymentQueue: SKPaymentQueue
    private let apiProxy: APIQuerying
    private let accountsProxy: RESTAccountHandling
    private var observerList = ObserverList<StorePaymentObserver>()
    private let transactionLog: StoreTransactionLog

    /// Payment manager's delegate.
    weak var delegate: StorePaymentManagerDelegate?

    /// A dictionary that maps each payment to account number.
    private var paymentToAccountToken = [SKPayment: String]()

    /// Returns true if the device is able to make payments.
    class var canMakePayments: Bool {
        SKPaymentQueue.canMakePayments()
    }

    /// Designated initializer
    ///
    /// - Parameters:
    ///   - backgroundTaskProvider: the background task provider.
    ///   - queue: the payment queue. Typically `SKPaymentQueue.default()`.
    ///   - apiProxy: the object implement `APIQuerying`
    ///   - accountsProxy: the object implementing `RESTAccountHandling`.
    ///   - transactionLog: an instance of transaction log. Typically ``StoreTransactionLog/default``.
    init(
        backgroundTaskProvider: BackgroundTaskProvider,
        queue: SKPaymentQueue,
        apiProxy: APIQuerying,
        accountsProxy: RESTAccountHandling,
        transactionLog: StoreTransactionLog
    ) {
        self.backgroundTaskProvider = backgroundTaskProvider
        paymentQueue = queue
        self.apiProxy = apiProxy
        self.accountsProxy = accountsProxy
        self.transactionLog = transactionLog
    }

    /// Loads transaction log from disk and starts monitoring payment queue.
    func start() {
        // Load transaction log from file before starting the payment queue.
        logger.debug("Load transaction log.")
        transactionLog.read()

        logger.debug("Start payment queue monitoring")
        paymentQueue.add(self)
    }

    // MARK: - SKPaymentTransactionObserver

    func paymentQueue(_ queue: SKPaymentQueue, updatedTransactions transactions: [SKPaymentTransaction]) {
        // Ensure that all calls happen on main queue because StoreKit does not guarantee on which queue the delegate
        // will be invoked.
        DispatchQueue.main.async {
            self.handleTransactions(transactions)
        }
    }

    // MARK: - Payment observation

    /// Add payment observer
    /// - Parameter observer: an observer object.
    func addPaymentObserver(_ observer: StorePaymentObserver) {
        observerList.append(observer)
    }

    /// Remove payment observer
    /// - Parameter observer: an observer object.
    func removePaymentObserver(_ observer: StorePaymentObserver) {
        observerList.remove(observer)
    }

    // MARK: - Products and payments

    /// Fetch products from AppStore using product identifiers.
    ///
    /// - Parameters:
    ///   - productIdentifiers: a set of product identifiers.
    ///   - completionHandler: completion handler. Invoked on main queue.
    /// - Returns: the request cancellation token
    func requestProducts(
        with productIdentifiers: Set<StoreSubscription>,
        completionHandler: @escaping (Result<SKProductsResponse, Error>) -> Void
    ) -> Cancellable {
        let productIdentifiers = productIdentifiers.productIdentifiersSet
        let operation = ProductsRequestOperation(
            productIdentifiers: productIdentifiers,
            completionHandler: completionHandler
        )
        operation.addCondition(MutuallyExclusive(category: OperationCategory.productsRequest))

        operationQueue.addOperation(operation)

        return operation
    }

    /// Add payment and associate it with the account number.
    ///
    /// Validates the user account with backend before adding the payment to the queue.
    ///
    /// - Parameters:
    ///   - payment: an intance of `SKPayment`.
    ///   - accountNumber: the account number to credit.
    func addPayment(_ payment: SKPayment, for accountNumber: String) {
        logger.debug("Validating account before the purchase.")

        // Validate account token before adding new payment to the queue.
        validateAccount(accountNumber: accountNumber) { error in
            if let error {
                self.logger.error("Failed to validate the account. Payment is ignored.")

                let event = StorePaymentEvent.failure(
                    StorePaymentFailure(
                        transaction: nil,
                        payment: payment,
                        accountNumber: accountNumber,
                        error: error
                    )
                )

                self.observerList.forEach { observer in
                    observer.storePaymentManager(self, didReceiveEvent: event)
                }
            } else {
                self.logger.debug("Add payment to the queue.")

                self.associateAccountNumber(accountNumber, and: payment)
                self.paymentQueue.add(payment)
            }
        }
    }

    /// Restore purchases by sending the AppStore receipt to backend.
    ///
    /// - Parameters:
    ///   - accountNumber: the account number to credit.
    ///   - completionHandler: completion handler invoked on the main queue.
    /// - Returns: the request cancellation token.
    func restorePurchases(
        for accountNumber: String,
        completionHandler: @escaping (Result<REST.CreateApplePaymentResponse, Error>) -> Void
    ) -> Cancellable {
        logger.debug("Restore purchases.")

        return sendStoreReceipt(
            accountNumber: accountNumber,
            forceRefresh: true,
            completionHandler: completionHandler
        )
    }

    // MARK: - Private methods

    /// Associate account number with the payment object.
    ///
    /// - Parameters:
    ///   - accountNumber: the account number that should be credited with the payment.
    ///   - payment: the payment object.
    private func associateAccountNumber(_ accountNumber: String, and payment: SKPayment) {
        dispatchPrecondition(condition: .onQueue(.main))

        paymentToAccountToken[payment] = accountNumber
    }

    /// Remove association between the payment object and the account number.
    ///
    /// Since the association between account numbers and payments is not persisted, this method may consult the delegate to provide the account number to
    /// credit. This can happen for dangling transactions that remain in the payment queue between the application restarts. In the future this association should be
    /// solved by using `SKPaymentQueue.applicationUsername`.
    ///
    /// - Parameter payment: the payment object.
    /// - Returns: The account number on success, otherwise `nil`.
    private func deassociateAccountNumber(_ payment: SKPayment) -> String? {
        dispatchPrecondition(condition: .onQueue(.main))

        if let accountToken = paymentToAccountToken[payment] {
            paymentToAccountToken.removeValue(forKey: payment)
            return accountToken
        } else {
            return delegate?.storePaymentManager(self, didRequestAccountTokenFor: payment)
        }
    }

    /// Validate account number.
    ///
    /// - Parameters:
    ///   - accountNumber: the account number
    ///   - completionHandler: completion handler invoked on main queue. The completion block Receives `nil` upon success, otherwise an error.
    private func validateAccount(
        accountNumber: String,
        completionHandler: @escaping (StorePaymentManagerError?) -> Void
    ) {
        let accountOperation = ResultBlockOperation<Account>(dispatchQueue: .main) { finish in
            self.accountsProxy.getAccountData(
                accountNumber: accountNumber,
                retryStrategy: .default,
                completion: finish
            )
        }

        accountOperation.addObserver(BackgroundObserver(
            application: backgroundTaskProvider,
            name: "Validate account number",
            cancelUponExpiration: false
        ))

        accountOperation.completionQueue = .main
        accountOperation.completionHandler = { result in
            completionHandler(result.error.map { StorePaymentManagerError.validateAccount($0) })
        }

        operationQueue.addOperation(accountOperation)
    }

    /// Send the AppStore receipt stored on device to the backend.
    ///
    /// - Parameters:
    ///   - accountNumber: the account number to credit.
    ///   - forceRefresh: indicates whether the receipt should be downloaded from AppStore even when it's present on device.
    ///   - completionHandler: a completion handler invoked on main queue.
    /// - Returns: the request cancellation token.
    private func sendStoreReceipt(
        accountNumber: String,
        forceRefresh: Bool,
        completionHandler: @escaping (Result<REST.CreateApplePaymentResponse, Error>) -> Void
    ) -> Cancellable {
        let operation = SendStoreReceiptOperation(
            apiProxy: apiProxy,
            accountNumber: accountNumber,
            forceRefresh: forceRefresh,
            receiptProperties: nil,
            completionHandler: completionHandler
        )

        operation.addObserver(
            BackgroundObserver(
                application: backgroundTaskProvider,
                name: "Send AppStore receipt",
                cancelUponExpiration: true
            )
        )

        operation.addCondition(MutuallyExclusive(category: OperationCategory.sendStoreReceipt))

        operationQueue.addOperation(operation)

        return operation
    }

    /// Handles an array of StoreKit transactions.
    /// - Parameter transactions: an array of transactions
    private func handleTransactions(_ transactions: [SKPaymentTransaction]) {
        transactions.forEach { transaction in
            handleTransaction(transaction)
        }
    }

    /// Handle single StoreKit transaction.
    /// - Parameter transaction: a transaction
    private func handleTransaction(_ transaction: SKPaymentTransaction) {
        switch transaction.transactionState {
        case .deferred:
            logger.info("Deferred \(transaction.payment.productIdentifier)")

        case .failed:
            let transactionError = transaction.error?.localizedDescription ?? "No error"
            logger.error("Failed to purchase \(transaction.payment.productIdentifier): \(transactionError)")

            didFailPurchase(transaction: transaction)

        case .purchased:
            logger.info("Purchased \(transaction.payment.productIdentifier)")

            didFinishOrRestorePurchase(transaction: transaction)

        case .purchasing:
            logger.info("Purchasing \(transaction.payment.productIdentifier)")

        case .restored:
            logger.info("Restored \(transaction.payment.productIdentifier)")

            didFinishOrRestorePurchase(transaction: transaction)

        @unknown default:
            logger.warning("Unknown transactionState = \(transaction.transactionState.rawValue)")
        }
    }

    /// Handle failed transaction by finishing it and notifying the observers.
    ///
    /// - Parameter transaction: the failed transaction.
    private func didFailPurchase(transaction: SKPaymentTransaction) {
        paymentQueue.finishTransaction(transaction)

        let paymentFailure = if let accountToken = deassociateAccountNumber(transaction.payment) {
            StorePaymentFailure(
                transaction: transaction,
                payment: transaction.payment,
                accountNumber: accountToken,
                error: .storePayment(transaction.error!)
            )
        } else {
            StorePaymentFailure(
                transaction: transaction,
                payment: transaction.payment,
                accountNumber: nil,
                error: .noAccountSet
            )
        }

        observerList.forEach { observer in
            observer.storePaymentManager(self, didReceiveEvent: .failure(paymentFailure))
        }
    }

    /// Handle successful transaction that's in purchased or restored state.
    ///
    /// - Consults with transaction log before handling the transaction. Transactions that are already processed are removed from the payment queue,
    ///   observers are not notified as they had already received the corresponding events.
    /// - Keeps transaction in the queue if association between transaction payment and account number cannot be established. Notifies observers with the error.
    /// - Sends the AppStore receipt to backend.
    ///
    /// - Parameter transaction: the transaction that's in purchased or restored state.
    private func didFinishOrRestorePurchase(transaction: SKPaymentTransaction) {
        // Obtain transaction identifier which must be set on transactions with purchased or restored state.
        guard let transactionIdentifier = transaction.transactionIdentifier else {
            logger.warning("Purchased or restored transaction does not contain a transaction identifier!")
            return
        }

        // Check if transaction is already processed.
        guard !transactionLog.contains(transactionIdentifier: transactionIdentifier) else {
            logger.debug("Found transaction that is already processed.")
            paymentQueue.finishTransaction(transaction)
            return
        }

        // Find the account number associated with the payment.
        guard let accountNumber = deassociateAccountNumber(transaction.payment) else {
            logger.debug("Cannot locate the account associated with the purchase. Keep transaction in the queue.")

            let event = StorePaymentEvent.failure(
                StorePaymentFailure(
                    transaction: transaction,
                    payment: transaction.payment,
                    accountNumber: nil,
                    error: .noAccountSet
                )
            )

            observerList.forEach { observer in
                observer.storePaymentManager(self, didReceiveEvent: event)
            }
            return
        }

        // Send the AppStore receipt to the backend.
        _ = sendStoreReceipt(accountNumber: accountNumber, forceRefresh: false) { result in
            self.didSendStoreReceipt(
                accountNumber: accountNumber,
                transactionIdentifier: transactionIdentifier,
                transaction: transaction,
                result: result
            )
        }
    }

    /// Handles the result of uploading the AppStore receipt to the backend.
    ///
    /// If the server response is successful, this function adds the transaction identifier to the transaction log to make sure that the same transaction is not
    /// processed twice, then finishes the transaction.
    ///
    /// This is important because the call to `SKPaymentQueue.finishTransaction()` may fail, causing the same transaction to re-appear on the payment
    /// queue. Since the transaction was already processed, no action needs to be performed besides another attempt to finish it and hopefully remove it from
    /// the payment queue for good.
    ///
    /// If the server response indicates an error, then this function keeps the transaction in the payment queue in order to process it again later.
    ///
    /// Finally, the ``StorePaymentEvent`` is produced and dispatched to observers to notify them on the progress.
    ///
    /// - Parameters:
    ///   - accountNumber: the account number to credit
    ///   - transactionIdentifier: the transaction identifier
    ///   - transaction: the transaction object
    ///   - result: the result of uploading the AppStore receipt to the backend.
    private func didSendStoreReceipt(
        accountNumber: String,
        transactionIdentifier: String,
        transaction: SKPaymentTransaction,
        result: Result<REST.CreateApplePaymentResponse, Error>
    ) {
        var event: StorePaymentEvent?

        switch result {
        case let .success(response):
            // Save transaction identifier to transaction log to identify it later if it resurrects on the payment queue.
            transactionLog.add(transactionIdentifier: transactionIdentifier)

            // Finish transaction to remove it from the payment queue.
            paymentQueue.finishTransaction(transaction)

            event = StorePaymentEvent.finished(StorePaymentCompletion(
                transaction: transaction,
                accountNumber: accountNumber,
                serverResponse: response
            ))

        case let .failure(error as StorePaymentManagerError):
            logger.debug("Failed to upload the receipt. Keep transaction in the queue.")

            event = StorePaymentEvent.failure(StorePaymentFailure(
                transaction: transaction,
                payment: transaction.payment,
                accountNumber: accountNumber,
                error: error
            ))

        default:
            break
        }

        if let event {
            observerList.forEach { observer in
                observer.storePaymentManager(self, didReceiveEvent: event)
            }
        }
    }
}

// swiftlint:disable:this file_length
