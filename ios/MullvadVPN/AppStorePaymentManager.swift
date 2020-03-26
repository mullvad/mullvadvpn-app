//
//  AppStorePaymentManager.swift
//  MullvadVPN
//
//  Created by pronebird on 10/03/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation
import StoreKit
import os

enum InAppPurchase: String {
    /// Thirty days worth of credit
    case thirtyDays = "net.mullvad.MullvadVPN.iap.30days"
}

extension Set where Element == InAppPurchase {
    var productIdentifiersSet: Set<String> {
        Set<String>(self.map { $0.rawValue })
    }
}

protocol AppStorePaymentObserver: class {
    func appStorePaymentManager(
        _ manager: AppStorePaymentManager,
        transaction: SKPaymentTransaction,
        didFailWithError error: AppStorePaymentManager.Error)

    func appStorePaymentManager(
        _ manager: AppStorePaymentManager,
        transaction: SKPaymentTransaction,
        didFinishWithResponse response: SendAppStoreReceiptResponse)
}

/// A type-erasing weak container for `AppStorePaymentObserver`
private class WeakAnyAppStorePaymentObserver: AppStorePaymentObserver {
    private(set) weak var inner: AppStorePaymentObserver?

    init(_ inner: AppStorePaymentObserver) {
        self.inner = inner
    }

    func appStorePaymentManager(_ manager: AppStorePaymentManager,
                                transaction: SKPaymentTransaction,
                                didFailWithError error: AppStorePaymentManager.Error)
    {
        inner?.appStorePaymentManager(manager, transaction: transaction, didFailWithError: error)
    }

    func appStorePaymentManager(_ manager: AppStorePaymentManager,
                                transaction: SKPaymentTransaction,
                                didFinishWithResponse response: SendAppStoreReceiptResponse)
    {
        inner?.appStorePaymentManager(manager,
                                      transaction: transaction,
                                      didFinishWithResponse: response)
    }

}

protocol AppStorePaymentManagerDelegate: class {

    /// Return the account token associated with the payment.
    /// Usually called for unfinished transactions coming back after the app was restarted.
    func appStorePaymentManager(_ manager: AppStorePaymentManager,
                                didRequestAccountTokenFor payment: SKPayment) -> String?
}

class AppStorePaymentManager {

    enum SendAppStoreReceiptError: Swift.Error {
        case read(AppStoreReceipt.Error)
        case network(MullvadAPI.Error)
        case server(MullvadAPI.ResponseError)
    }

    enum Error: Swift.Error {
        case noAccountSet
        case storePayment(Swift.Error)
        case sendReceipt(SendAppStoreReceiptError)
    }

    /// A shared instance of `AppStorePaymentManager`
    static let shared = AppStorePaymentManager(queue: SKPaymentQueue.default())

    private let queue: SKPaymentQueue
    private let apiClient = MullvadAPI()

    private var paymentQueueSubscriber: AnyCancellable?
    private var sendReceiptSubscriber: AnyCancellable?

    private var observers = [WeakAnyAppStorePaymentObserver]()
    private let lock = NSRecursiveLock()

    private weak var classDelegate: AppStorePaymentManagerDelegate?
    weak var delegate: AppStorePaymentManagerDelegate? {
        get {
            lock.withCriticalBlock {
                return classDelegate
            }
        }
        set {
            lock.withCriticalBlock {
                classDelegate = newValue
            }
        }
    }

    /// A private hash map that maps each payment to account token
    private var paymentToAccountToken = [SKPayment: String]()

    init(queue: SKPaymentQueue) {
        self.queue = queue
    }

    func startPaymentQueueMonitoring() {
        paymentQueueSubscriber = queue.publisher.sink { [weak self] (transaction) in
            self?.handleTransaction(transaction)
        }
    }

    // MARK: - Payment observation

    func addPaymentObserver(_ observer: AppStorePaymentObserver) {
        lock.withCriticalBlock {
            let isAlreadyObserving = self.observers.contains(where: { $0.inner === observer })

            if !isAlreadyObserving {
                self.observers.append(WeakAnyAppStorePaymentObserver(observer))
                self.compactObservers()
            }
        }
    }

    func removePaymentObserver(_ observer: AppStorePaymentObserver) {
        lock.withCriticalBlock {
            let index = self.observers.firstIndex(where: { $0.inner === observer })
            if let index = index {
                self.observers.remove(at: index)
            }
        }
    }

    private func compactObservers() {
        lock.withCriticalBlock {
            observers.removeAll(where: { $0.inner == nil })
        }
    }

    private func enumerateObservers(_ body: (AppStorePaymentObserver) -> Void) {
        lock.withCriticalBlock {
            observers.forEach { (observer) in
                body(observer)
            }
        }
    }

    // MARK: - Account token and payment mapping

    private func associateAccountToken(_ token: String, and payment: SKPayment) {
        lock.withCriticalBlock {
            paymentToAccountToken[payment] = token
        }
    }

    private func deassociateAccountToken(_ payment: SKPayment) -> String? {
        return lock.withCriticalBlock {
            if let accountToken = paymentToAccountToken[payment] {
                paymentToAccountToken.removeValue(forKey: payment)
                return accountToken
            } else {
                return self.classDelegate?
                    .appStorePaymentManager(self, didRequestAccountTokenFor: payment)
            }
        }
    }

    // MARK: - Products and payments

    func requestProducts(with productIdentifiers: Set<InAppPurchase>)
        -> SKRequestPublisher<SKProductsRequestSubscription>
    {
        let productIdentifiers = productIdentifiers.productIdentifiersSet

        return SKProductsRequest(productIdentifiers: productIdentifiers).publisher
    }

    func addPayment(_ payment: SKPayment, for accountToken: String) -> AppStorePaymentPublisher {
        associateAccountToken(accountToken, and: payment)

        return AppStorePaymentPublisher(paymentManager: self, queue: queue, payment: payment)
    }

    func restorePurchases(for accountToken: String) -> AnyPublisher<SendAppStoreReceiptResponse, AppStorePaymentManager.Error> {
        return sendAppStoreReceipt(accountToken: accountToken, forceRefresh: true)
    }

    // MARK: - Private methods

    private func sendAppStoreReceipt(accountToken: String, forceRefresh: Bool) ->
        AnyPublisher<SendAppStoreReceiptResponse, AppStorePaymentManager.Error>
    {
        return AppStoreReceipt.fetch(forceRefresh: forceRefresh)
            .mapError { SendAppStoreReceiptError.read($0) }
            .flatMap { (receiptData) in
                self.apiClient.sendAppStoreReceipt(accountToken: accountToken, receiptData: receiptData)
                    .mapError { SendAppStoreReceiptError.network($0) }
                    .flatMap({ (response) in
                        response.result.mapError { SendAppStoreReceiptError.server($0) }.publisher
                    })
        }
        .receive(on: DispatchQueue.main)
        .handleEvents(receiveOutput: { (response) in
            os_log(
                .info,
                "AppStore Receipt was processed. Time added: %{public}.2f, New expiry: %{public}s",
                response.timeAdded, "\(response.newExpiry)")
        })
            .mapError { AppStorePaymentManager.Error.sendReceipt($0) }
            .eraseToAnyPublisher()
    }

    private func handleTransaction(_ transaction: SKPaymentTransaction) {
        switch transaction.transactionState {
        case .deferred:
            os_log(.debug, "Deferred %{public}s", transaction.payment.productIdentifier)

        case .failed:
            os_log(.debug, "Failed to purchase %{public}s: %{public}s",
                   transaction.payment.productIdentifier,
                   transaction.error?.localizedDescription ?? "No error")

            didFailPurchase(transaction: transaction)

        case .purchased:
            os_log(.debug, "Purchased %{public}s", transaction.payment.productIdentifier)

            didFinishOrRestorePurchase(transaction: transaction)

        case .purchasing:
            os_log(.debug, "Purchasing %{public}s", transaction.payment.productIdentifier)

        case .restored:
            os_log(.debug, "Restored %{public}s", transaction.payment.productIdentifier)

            didFinishOrRestorePurchase(transaction: transaction)

        @unknown default:
            os_log(.debug, "Unknown transactionState = %{public}d",
                   transaction.transactionState.rawValue)
        }
    }

    private func didFailPurchase(transaction: SKPaymentTransaction) {
        queue.finishTransaction(transaction)

        enumerateObservers { (observer) in
            observer.appStorePaymentManager(
                self,
                transaction: transaction,
                didFailWithError: .storePayment(transaction.error!))
        }

        _ = deassociateAccountToken(transaction.payment)
    }

    private func didFinishOrRestorePurchase(transaction: SKPaymentTransaction) {
        let accountToken = deassociateAccountToken(transaction.payment)

        sendReceiptSubscriber = Just(accountToken)
            .setFailureType(to: AppStorePaymentManager.Error.self)
            .replaceNil(with: .noAccountSet)
            .flatMap({ (accountToken) in
                self.sendAppStoreReceipt(accountToken: accountToken, forceRefresh: false)
                    .retry(1)
            })
            .receive(on: DispatchQueue.main)
            .sink(receiveCompletion: { [weak self] (completion) in
                guard let self = self else { return }

                switch completion {
                case .finished:
                    self.queue.finishTransaction(transaction)

                case .failure(let error):
                    os_log(.error, "Failed to upload the AppStore receipt: %{public}s",
                           error.localizedDescription)

                    self.enumerateObservers { (observer) in
                        observer.appStorePaymentManager(
                            self,
                            transaction: transaction,
                            didFailWithError: error)
                    }
                }
            }, receiveValue: { [weak self] (response) in
                guard let self = self else { return }

                self.enumerateObservers { (observer) in
                    observer.appStorePaymentManager(
                        self,
                        transaction: transaction,
                        didFinishWithResponse: response)
                }
            })
    }

}


extension AppStorePaymentManager.Error: LocalizedError {

    var errorDescription: String? {
        switch self {
        case .noAccountSet:
            return nil
        case .storePayment:
            return NSLocalizedString("AppStore payment", comment: "")
        case .sendReceipt:
            return NSLocalizedString("Communication error", comment: "")
        }
    }

    var failureReason: String? {
        switch self {
        case .storePayment(let storeError):
            return storeError.localizedDescription
        case .sendReceipt(.network(let urlError)):
            return urlError.localizedDescription
        case .sendReceipt(.server(let serverError)):
            return serverError.errorDescription
        case .sendReceipt(.read(.refresh(let storeError))):
            return storeError.localizedDescription
        default:
            return NSLocalizedString("Internal error", comment: "")
        }
    }

    var recoverySuggestion: String? {
        switch self {
        case .noAccountSet:
            return nil
        case .storePayment:
            return nil
        case .sendReceipt:
            return NSLocalizedString(
                #"Please retry by using the "Restore purchases" button"#, comment: "")
        }
    }
}
