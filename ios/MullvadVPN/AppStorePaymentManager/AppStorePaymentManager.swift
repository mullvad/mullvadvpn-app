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

    private let exclusivityController = ExclusivityController()

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

    func addPaymentObserver(_ observer: AppStorePaymentObserver) {
        observerList.append(observer)
    }

    func removePaymentObserver(_ observer: AppStorePaymentObserver) {
        observerList.remove(observer)
    }

    // MARK: - Products and payments

    func requestProducts(with productIdentifiers: Set<AppStoreSubscription>, completionHandler: @escaping (OperationCompletion<SKProductsResponse, Swift.Error>) -> Void) -> Cancellable {
        let productIdentifiers = productIdentifiers.productIdentifiersSet
        let operation = ProductsRequestOperation(productIdentifiers: productIdentifiers, completionHandler: completionHandler)

        exclusivityController.addOperation(operation, categories: [OperationCategory.productsRequest])

        operationQueue.addOperation(operation)

        return operation
    }

    func addPayment(_ payment: SKPayment, for accountToken: String) {
        var cancellableTask: Cancellable?
        let backgroundTaskIdentifier = UIApplication.shared.beginBackgroundTask(withName: "Validate account token") {
            cancellableTask?.cancel()
        }

        // Validate account token before adding new payment to the queue.
        cancellableTask = REST.Client.shared.getAccountExpiry(token: accountToken, retryStrategy: .default) { result in
            dispatchPrecondition(condition: .onQueue(.main))

            switch result {
            case .success:
                self.associateAccountToken(accountToken, and: payment)
                self.paymentQueue.add(payment)

            case .failure(let error):
                self.observerList.forEach { observer in
                    observer.appStorePaymentManager(
                        self,
                        transaction: nil,
                        payment: payment,
                        accountToken: accountToken,
                        didFailWithError: .validateAccount(error)
                    )
                }
            }

            UIApplication.shared.endBackgroundTask(backgroundTaskIdentifier)
        }
    }

    func restorePurchases(for accountToken: String, completionHandler: @escaping (OperationCompletion<REST.CreateApplePaymentResponse, AppStorePaymentManager.Error>) -> Void) -> Cancellable {
        return sendAppStoreReceipt(accountToken: accountToken, forceRefresh: true, completionHandler: completionHandler)
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

    private func sendAppStoreReceipt(accountToken: String, forceRefresh: Bool, completionHandler: @escaping (OperationCompletion<REST.CreateApplePaymentResponse, Error>) -> Void) -> Cancellable {
        let operation = SendAppStoreReceiptOperation(restClient: REST.Client.shared, accountToken: accountToken, forceRefresh: forceRefresh, receiptProperties: nil) { completion in
            completionHandler(completion)
        }

        let backgroundTaskIdentifier = UIApplication.shared.beginBackgroundTask(withName: "Send AppStore receipt") {
            operation.cancel()
        }

        operation.completionBlock = {
            UIApplication.shared.endBackgroundTask(backgroundTaskIdentifier)
        }

        exclusivityController.addOperation(operation, categories: [OperationCategory.sendAppStoreReceipt])

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
            observerList.forEach { observer in
                observer.appStorePaymentManager(
                    self,
                    transaction: transaction,
                    payment: transaction.payment,
                    accountToken: accountToken,
                    didFailWithError: .storePayment(transaction.error!))
            }
        } else {
            observerList.forEach { observer in
                observer.appStorePaymentManager(
                    self,
                    transaction: transaction,
                    payment: transaction.payment,
                    accountToken: nil,
                    didFailWithError: .noAccountSet)
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
                    didFailWithError: .noAccountSet)
            }
            return
        }

        _ = sendAppStoreReceipt(accountToken: accountToken, forceRefresh: false) { completion in
            switch completion {
            case .success(let response):
                self.paymentQueue.finishTransaction(transaction)

                self.observerList.forEach { observer in
                    observer.appStorePaymentManager(
                        self,
                        transaction: transaction,
                        accountToken: accountToken,
                        didFinishWithResponse: response)
                }

            case .failure(let error):
                self.observerList.forEach { observer in
                    observer.appStorePaymentManager(
                        self,
                        transaction: transaction,
                        payment: transaction.payment,
                        accountToken: accountToken,
                        didFailWithError: error)
                }

            case .cancelled:
                break
            }
        }
    }

}

private class SendAppStoreReceiptOperation: AsyncOperation {
    typealias CompletionHandler = (OperationCompletion<REST.CreateApplePaymentResponse, AppStorePaymentManager.Error>) -> Void

    private let restClient: REST.Client
    private let accountToken: String
    private let forceRefresh: Bool
    private let receiptProperties: [String: Any]?
    private var completionHandler: CompletionHandler?
    private var fetchReceiptCancellable: Cancellable?
    private var submitReceiptCancellable: Cancellable?

    private let logger = Logger(label: "AppStorePaymentManager.SendAppStoreReceiptOperation")

    init(restClient: REST.Client, accountToken: String, forceRefresh: Bool, receiptProperties: [String: Any]?, completionHandler: @escaping CompletionHandler) {
        self.restClient = restClient
        self.accountToken = accountToken
        self.forceRefresh = forceRefresh
        self.receiptProperties = receiptProperties
        self.completionHandler = completionHandler
    }

    override func cancel() {
        super.cancel()

        DispatchQueue.main.async {
            self.fetchReceiptCancellable?.cancel()
            self.fetchReceiptCancellable = nil

            self.submitReceiptCancellable?.cancel()
            self.submitReceiptCancellable = nil
        }
    }

    override func main() {
        DispatchQueue.main.async {
            self.fetchReceiptCancellable = AppStoreReceipt.fetch(forceRefresh: self.forceRefresh, receiptProperties: self.receiptProperties) { completion in
                switch completion {
                case .success(let receiptData):
                    self.sendReceipt(receiptData)

                case .failure(let error):
                    self.logger.error(chainedError: error, message: "Failed to fetch the AppStore receipt.")
                    self.finish(completion: .failure(.readReceipt(error)))

                case .cancelled:
                    self.finish(completion: .cancelled)
                }
            }
        }
    }

    private func sendReceipt(_ receiptData: Data) {
        submitReceiptCancellable = restClient.createApplePayment(
            token: self.accountToken,
            receiptString: receiptData,
            retryStrategy: .noRetry) { result in
                switch result {
                case .success(let response):
                    self.logger.info("AppStore receipt was processed. Time added: \(response.timeAdded), New expiry: \(response.newExpiry.logFormatDate())")
                    self.finish(completion: .success(response))

                case .failure(let error):
                    self.logger.error(chainedError: error, message: "Failed to send the AppStore receipt.")
                    self.finish(completion: .failure(.sendReceipt(error)))
                }
            }
    }

    private func finish(completion: OperationCompletion<REST.CreateApplePaymentResponse, AppStorePaymentManager.Error>) {
        let block = completionHandler
        completionHandler = nil

        block?(completion)
        finish()
    }
}
