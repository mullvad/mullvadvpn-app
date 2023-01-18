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

    private let application: UIApplication
    private let paymentQueue: SKPaymentQueue
    private let apiProxy: REST.APIProxy
    private let accountsProxy: REST.AccountsProxy
    private var observerList = ObserverList<StorePaymentObserver>()

    private weak var classDelegate: StorePaymentManagerDelegate?
    weak var delegate: StorePaymentManagerDelegate? {
        get {
            if Thread.isMainThread {
                return classDelegate
            } else {
                return DispatchQueue.main.sync {
                    return classDelegate
                }
            }
        }
        set {
            if Thread.isMainThread {
                classDelegate = newValue
            } else {
                DispatchQueue.main.async {
                    self.classDelegate = newValue
                }
            }
        }
    }

    /// A private hash map that maps each payment to account token.
    private var paymentToAccountToken = [SKPayment: String]()

    /// Returns true if the device is able to make payments.
    class var canMakePayments: Bool {
        return SKPaymentQueue.canMakePayments()
    }

    init(
        application: UIApplication,
        queue: SKPaymentQueue,
        apiProxy: REST.APIProxy,
        accountsProxy: REST.AccountsProxy
    ) {
        self.application = application
        paymentQueue = queue
        self.apiProxy = apiProxy
        self.accountsProxy = accountsProxy
    }

    func startPaymentQueueMonitoring() {
        logger.debug("Start payment queue monitoring")
        paymentQueue.add(self)
    }

    // MARK: - SKPaymentTransactionObserver

    func paymentQueue(
        _ queue: SKPaymentQueue,
        updatedTransactions transactions: [SKPaymentTransaction]
    ) {
        // Ensure that all calls happen on main queue
        if Thread.isMainThread {
            handleTransactions(transactions)
        } else {
            DispatchQueue.main.async {
                self.handleTransactions(transactions)
            }
        }
    }

    // MARK: - Payment observation

    func addPaymentObserver(_ observer: StorePaymentObserver) {
        observerList.append(observer)
    }

    func removePaymentObserver(_ observer: StorePaymentObserver) {
        observerList.remove(observer)
    }

    // MARK: - Products and payments

    func requestProducts(
        with productIdentifiers: Set<StoreSubscription>,
        completionHandler: @escaping (OperationCompletion<SKProductsResponse, Error>) -> Void
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

    func addPayment(_ payment: SKPayment, for accountNumber: String) {
        // Validate account token before adding new payment to the queue.
        validateAccount(accountNumber: accountNumber) { error in
            if let error = error {
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
                self.associateAccountToken(accountNumber, and: payment)
                self.paymentQueue.add(payment)
            }
        }
    }

    func restorePurchases(
        for accountNumber: String,
        completionHandler: @escaping (OperationCompletion<
            REST.CreateApplePaymentResponse,
            StorePaymentManagerError
        >) -> Void
    ) -> Cancellable {
        return sendStoreReceipt(
            accountNumber: accountNumber,
            forceRefresh: true,
            completionHandler: completionHandler
        )
    }

    // MARK: - Private methods

    private func associateAccountToken(_ token: String, and payment: SKPayment) {
        assert(Thread.isMainThread)

        paymentToAccountToken[payment] = token
    }

    private func deassociateAccountToken(_ payment: SKPayment) -> String? {
        assert(Thread.isMainThread)

        if let accountToken = paymentToAccountToken[payment] {
            paymentToAccountToken.removeValue(forKey: payment)
            return accountToken
        } else {
            return classDelegate?.storePaymentManager(self, didRequestAccountTokenFor: payment)
        }
    }

    private func validateAccount(
        accountNumber: String,
        completionHandler: @escaping (StorePaymentManagerError?) -> Void
    ) {
        let accountOperation = ResultBlockOperation<
            REST.AccountData,
            REST.Error
        >(dispatchQueue: .main) { op in
            let task = self.accountsProxy.getAccountData(
                accountNumber: accountNumber,
                retryStrategy: .default
            ) { completion in
                op.finish(completion: completion)
            }

            op.addCancellationBlock {
                task.cancel()
            }
        }

        accountOperation.addObserver(BackgroundObserver(
            application: application,
            name: "Validate account number",
            cancelUponExpiration: false
        ))

        accountOperation.completionQueue = .main
        accountOperation.completionHandler = { completion in
            var error: REST.Error?

            if case .cancelled = completion {
                error = .network(URLError(.cancelled))
            } else {
                error = completion.error
            }

            completionHandler(error.map { .validateAccount($0) })
        }

        operationQueue.addOperation(accountOperation)
    }

    private func sendStoreReceipt(
        accountNumber: String,
        forceRefresh: Bool,
        completionHandler: @escaping (OperationCompletion<
            REST.CreateApplePaymentResponse,
            StorePaymentManagerError
        >) -> Void
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
                application: application,
                name: "Send AppStore receipt",
                cancelUponExpiration: true
            )
        )

        operation.addCondition(MutuallyExclusive(category: OperationCategory.sendStoreReceipt))

        operationQueue.addOperation(operation)

        return operation
    }

    private func handleTransactions(_ transactions: [SKPaymentTransaction]) {
        transactions.forEach { transaction in
            handleTransaction(transaction)
        }
    }

    private func handleTransaction(_ transaction: SKPaymentTransaction) {
        switch transaction.transactionState {
        case .deferred:
            logger.info("Deferred \(transaction.payment.productIdentifier)")

        case .failed:
            logger
                .error(
                    "Failed to purchase \(transaction.payment.productIdentifier): \(transaction.error?.localizedDescription ?? "No error")"
                )

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

    private func didFailPurchase(transaction: SKPaymentTransaction) {
        paymentQueue.finishTransaction(transaction)

        let paymentFailure: StorePaymentFailure

        if let accountToken = deassociateAccountToken(transaction.payment) {
            paymentFailure = StorePaymentFailure(
                transaction: transaction,
                payment: transaction.payment,
                accountNumber: accountToken,
                error: .storePayment(transaction.error!)
            )
        } else {
            paymentFailure = StorePaymentFailure(
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

    private func didFinishOrRestorePurchase(transaction: SKPaymentTransaction) {
        guard let accountNumber = deassociateAccountToken(transaction.payment) else {
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

        _ = sendStoreReceipt(accountNumber: accountNumber, forceRefresh: false) { completion in
            var event: StorePaymentEvent?

            switch completion {
            case let .success(response):
                self.paymentQueue.finishTransaction(transaction)

                event = StorePaymentEvent.finished(StorePaymentCompletion(
                    transaction: transaction,
                    accountNumber: accountNumber,
                    serverResponse: response
                ))

            case let .failure(error):
                event = StorePaymentEvent.failure(StorePaymentFailure(
                    transaction: transaction,
                    payment: transaction.payment,
                    accountNumber: accountNumber,
                    error: error
                ))

            case .cancelled:
                break
            }

            if let event = event {
                self.observerList.forEach { observer in
                    observer.storePaymentManager(self, didReceiveEvent: event)
                }
            }
        }
    }
}
