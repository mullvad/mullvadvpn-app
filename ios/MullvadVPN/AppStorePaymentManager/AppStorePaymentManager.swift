//
//  AppStorePaymentManager.swift
//  MullvadVPN
//
//  Created by pronebird on 10/03/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadTypes
import Operations
import StoreKit

class AppStorePaymentManager: NSObject, SKPaymentTransactionObserver {
    private enum OperationCategory {
        static let sendAppStoreReceipt = "AppStorePaymentManager.sendAppStoreReceipt"
        static let productsRequest = "AppStorePaymentManager.productsRequest"
    }

    /// A shared instance of `AppStorePaymentManager`
    static let shared = AppStorePaymentManager(queue: SKPaymentQueue.default())

    private let logger = Logger(label: "AppStorePaymentManager")

    private let operationQueue: OperationQueue = {
        let queue = AsyncOperationQueue()
        queue.name = "AppStorePaymentManagerQueue"
        return queue
    }()
    private static let proxyFactory = REST.ProxyFactory(addressCacheStoreAccessLevel: .readWrite)
    private let apiProxy = proxyFactory.createAPIProxy()
    private let accountsProxy = proxyFactory.createAccountsProxy()

    private let paymentQueue: SKPaymentQueue
    private var observerList = ObserverList<AppStorePaymentObserver>()

    private weak var classDelegate: AppStorePaymentManagerDelegate?
    weak var delegate: AppStorePaymentManagerDelegate? {
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

    init(queue: SKPaymentQueue) {
        paymentQueue = queue
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

    func addPaymentObserver(_ observer: AppStorePaymentObserver) {
        observerList.append(observer)
    }

    func removePaymentObserver(_ observer: AppStorePaymentObserver) {
        observerList.remove(observer)
    }

    // MARK: - Products and payments

    func requestProducts(
        with productIdentifiers: Set<AppStoreSubscription>,
        completionHandler: @escaping (OperationCompletion<SKProductsResponse, Swift.Error>) -> Void
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

    func addPayment(_ payment: SKPayment, for accountToken: String) {
        var task: Cancellable?
        let backgroundTaskIdentifier = UIApplication.shared
            .beginBackgroundTask(withName: "Validate account token") {
                task?.cancel()
            }

        // Validate account token before adding new payment to the queue.
        task = accountsProxy.getAccountData(
            accountNumber: accountToken,
            retryStrategy: .default
        ) { completion in
            dispatchPrecondition(condition: .onQueue(.main))

            switch completion {
            case .success:
                self.associateAccountToken(accountToken, and: payment)
                self.paymentQueue.add(payment)

            case let .failure(error):
                self.observerList.forEach { observer in
                    observer.appStorePaymentManager(
                        self,
                        transaction: nil,
                        payment: payment,
                        accountToken: accountToken,
                        didFailWithError: .validateAccount(error)
                    )
                }

            case .cancelled:
                self.observerList.forEach { observer in
                    observer.appStorePaymentManager(
                        self,
                        transaction: nil,
                        payment: payment,
                        accountToken: accountToken,
                        didFailWithError: .validateAccount(.network(URLError(.cancelled)))
                    )
                }
            }

            UIApplication.shared.endBackgroundTask(backgroundTaskIdentifier)
        }
    }

    func restorePurchases(
        for accountToken: String,
        completionHandler: @escaping (OperationCompletion<
            REST.CreateApplePaymentResponse,
            AppStorePaymentManager.Error
        >) -> Void
    ) -> Cancellable {
        return sendAppStoreReceipt(
            accountToken: accountToken,
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
            return classDelegate?.appStorePaymentManager(self, didRequestAccountTokenFor: payment)
        }
    }

    private func sendAppStoreReceipt(
        accountToken: String,
        forceRefresh: Bool,
        completionHandler: @escaping (OperationCompletion<REST.CreateApplePaymentResponse, Error>)
            -> Void
    ) -> Cancellable {
        let operation = SendAppStoreReceiptOperation(
            apiProxy: apiProxy,
            accountToken: accountToken,
            forceRefresh: forceRefresh,
            receiptProperties: nil,
            completionHandler: completionHandler
        )

        operation.addObserver(
            BackgroundObserver(
                application: .shared,
                name: "Send AppStore receipt",
                cancelUponExpiration: true
            )
        )

        operation.addCondition(
            MutuallyExclusive(category: OperationCategory.sendAppStoreReceipt)
        )

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

        if let accountToken = deassociateAccountToken(transaction.payment) {
            observerList.forEach { observer in
                observer.appStorePaymentManager(
                    self,
                    transaction: transaction,
                    payment: transaction.payment,
                    accountToken: accountToken,
                    didFailWithError: .storePayment(transaction.error!)
                )
            }
        } else {
            observerList.forEach { observer in
                observer.appStorePaymentManager(
                    self,
                    transaction: transaction,
                    payment: transaction.payment,
                    accountToken: nil,
                    didFailWithError: .noAccountSet
                )
            }
        }
    }

    private func didFinishOrRestorePurchase(transaction: SKPaymentTransaction) {
        guard let accountToken = deassociateAccountToken(transaction.payment) else {
            observerList.forEach { observer in
                observer.appStorePaymentManager(
                    self,
                    transaction: transaction,
                    payment: transaction.payment,
                    accountToken: nil,
                    didFailWithError: .noAccountSet
                )
            }
            return
        }

        _ = sendAppStoreReceipt(accountToken: accountToken, forceRefresh: false) { completion in
            switch completion {
            case let .success(response):
                self.paymentQueue.finishTransaction(transaction)

                self.observerList.forEach { observer in
                    observer.appStorePaymentManager(
                        self,
                        transaction: transaction,
                        accountToken: accountToken,
                        didFinishWithResponse: response
                    )
                }

            case let .failure(error):
                self.observerList.forEach { observer in
                    observer.appStorePaymentManager(
                        self,
                        transaction: transaction,
                        payment: transaction.payment,
                        accountToken: accountToken,
                        didFailWithError: error
                    )
                }

            case .cancelled:
                break
            }
        }
    }
}
