//
//  AppStorePaymentManager.swift
//  MullvadVPN
//
//  Created by pronebird on 10/03/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import StoreKit
import Logging

class AppStorePaymentManager: NSObject, SKPaymentTransactionObserver {

    private enum OperationCategory {
        static let sendAppStoreReceipt = "AppStorePaymentManager.sendAppStoreReceipt"
        static let productsRequest = "AppStorePaymentManager.productsRequest"
    }

    /// A shared instance of `AppStorePaymentManager`
    static let shared = AppStorePaymentManager(queue: SKPaymentQueue.default())

    private let logger = Logger(label: "AppStorePaymentManager")

    private let operationQueue: OperationQueue = {
        let queue = OperationQueue()
        queue.name = "AppStorePaymentManagerQueue"
        return queue
    }()

    private let paymentQueue: SKPaymentQueue
    private var observerList = ObserverList<AnyAppStorePaymentObserver>()

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

    /// A private hash map that maps each payment to account token
    private var paymentToAccountToken = [SKPayment: String]()

    /// Returns true if the device is able to make payments
    class var canMakePayments: Bool {
        return SKPaymentQueue.canMakePayments()
    }

    init(queue: SKPaymentQueue) {
        self.paymentQueue = queue
    }

    func startPaymentQueueMonitoring() {
        self.logger.debug("Start payment queue monitoring")
        paymentQueue.add(self)
    }

    // MARK: - SKPaymentTransactionObserver

    func paymentQueue(_ queue: SKPaymentQueue, updatedTransactions transactions: [SKPaymentTransaction]) {
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

    func addPaymentObserver<T: AppStorePaymentObserver>(_ observer: T) {
        observerList.append(AnyAppStorePaymentObserver(observer))
    }

    func removePaymentObserver<T: AppStorePaymentObserver>(_ observer: T) {
        observerList.remove(AnyAppStorePaymentObserver(observer))
    }

    // MARK: - Products and payments

    func requestProducts(with productIdentifiers: Set<AppStoreSubscription>) -> Result<SKProductsResponse, Swift.Error>.Promise {
        return Promise { resolver in
            let productIdentifiers = productIdentifiers.productIdentifiersSet
            let operation = ProductsRequestOperation(productIdentifiers: productIdentifiers) { result in
                resolver.resolve(value: result)
            }

            resolver.setCancelHandler {
                operation.cancel()
            }

            ExclusivityController.shared.addOperation(operation, categories: [OperationCategory.productsRequest])
            self.operationQueue.addOperation(operation)
        }
    }

    func addPayment(_ payment: SKPayment, for accountToken: String) {
        if Thread.isMainThread {
            _addPayment(payment, for: accountToken)
        } else {
            DispatchQueue.main.async {
                self._addPayment(payment, for: accountToken)
            }
        }
    }

    func restorePurchases(for accountToken: String) -> Result<REST.CreateApplePaymentResponse, AppStorePaymentManager.Error>.Promise {
        return sendAppStoreReceipt(accountToken: accountToken, forceRefresh: true)
            .requestBackgroundTime(taskName: "AppStorePaymentManager.restorePurchases")
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

    private func _addPayment(_ payment: SKPayment, for accountToken: String) {
        assert(Thread.isMainThread)

        associateAccountToken(accountToken, and: payment)
        paymentQueue.add(payment)
    }

    private func sendAppStoreReceipt(accountToken: String, forceRefresh: Bool) -> Result<REST.CreateApplePaymentResponse, Error>.Promise {
        return AppStoreReceipt.fetch(forceRefresh: forceRefresh)
            .mapError { error in
                self.logger.error(chainedError: error, message: "Failed to fetch the AppStore receipt")

                return .readReceipt(error)
            }
            .mapThen { receiptData in
                return REST.Client.shared.createApplePayment(token: accountToken, receiptString: receiptData)
                    .execute()
                    .mapError { error in
                        self.logger.error(chainedError: error, message: "Failed to upload the AppStore receipt")

                        return .sendReceipt(error)
                    }
                    .onSuccess{ response in
                        self.logger.info("AppStore receipt was processed. Time added: \(response.timeAdded), New expiry: \(response.newExpiry.logFormatDate())")
                    }
            }
            .run(on: operationQueue, categories: [OperationCategory.sendAppStoreReceipt])
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
            logger.error("Failed to purchase \(transaction.payment.productIdentifier): \(transaction.error?.localizedDescription ?? "No error")")

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
            observerList.forEach { (observer) in
                observer.appStorePaymentManager(
                    self,
                    transaction: transaction,
                    accountToken: accountToken,
                    didFailWithError: .storePayment(transaction.error!))
            }
        } else {
            observerList.forEach { (observer) in
                observer.appStorePaymentManager(
                    self,
                    transaction: transaction,
                    accountToken: nil,
                    didFailWithError: .noAccountSet)
            }
        }
    }

    private func didFinishOrRestorePurchase(transaction: SKPaymentTransaction) {
        if let accountToken = deassociateAccountToken(transaction.payment) {
            sendAppStoreReceipt(accountToken: accountToken, forceRefresh: false)
                .receive(on: .main)
                .onSuccess { response in
                    self.paymentQueue.finishTransaction(transaction)

                    self.observerList.forEach { (observer) in
                        observer.appStorePaymentManager(
                            self,
                            transaction: transaction,
                            accountToken: accountToken,
                            didFinishWithResponse: response)
                    }
                }
                .onFailure { error in
                    self.observerList.forEach { (observer) in
                        observer.appStorePaymentManager(
                            self,
                            transaction: transaction,
                            accountToken: accountToken,
                            didFailWithError: error)
                    }
                }
                .requestBackgroundTime(taskName: "AppStorePaymentManager.didFinishOrRestorePurchase")
                .observe { _ in }
        } else {
            observerList.forEach { (observer) in
                observer.appStorePaymentManager(
                    self,
                    transaction: transaction,
                    accountToken: nil,
                    didFailWithError: .noAccountSet)
            }
        }
    }

}
