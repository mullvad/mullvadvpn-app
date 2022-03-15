//
//  Account.swift
//  MullvadVPN
//
//  Created by pronebird on 16/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import StoreKit
import Logging

/// A enum holding the `UserDefaults` string keys
private enum UserDefaultsKeys: String {
    case isAgreedToTermsOfService = "isAgreedToTermsOfService"
    case accountToken = "accountToken"
    case accountExpiry = "accountExpiry"
}

protocol AccountObserver: AnyObject {
    func account(_ account: Account, didUpdateExpiry expiry: Date)
    func account(_ account: Account, didLoginWithToken token: String, expiry: Date)
    func accountDidLogout(_ account: Account)
}

/// A class that groups the account related operations
class Account {

    enum Error: ChainedError {
        /// A failure to create the new account token
        case createAccount(REST.Error)

        /// A failure to verify the account token
        case verifyAccount(REST.Error)

        /// A failure to configure a tunnel
        case tunnelConfiguration(TunnelManager.Error)

        var errorDescription: String? {
            switch self {
            case .createAccount:
                return "Failure to create new account."
            case .verifyAccount:
                return "Failure to verify account."
            case .tunnelConfiguration:
                return "Failure to configure the tunnel."
            }
        }
    }

    /// A shared instance of `Account`
    static let shared = Account()

    private let logger = Logger(label: "Account")
    private var observerList = ObserverList<AccountObserver>()

    private let operationQueue: OperationQueue = {
        let operationQueue = OperationQueue()
        operationQueue.maxConcurrentOperationCount = 1
        return operationQueue
    }()

    /// Returns true if user agreed to terms of service, otherwise false
    var isAgreedToTermsOfService: Bool {
        return UserDefaults.standard.bool(forKey: UserDefaultsKeys.isAgreedToTermsOfService.rawValue)
    }

    /// Returns the currently used account token
    private(set) var token: String? {
        set {
            UserDefaults.standard.set(newValue, forKey: UserDefaultsKeys.accountToken.rawValue)
        }
        get {
            return UserDefaults.standard.string(forKey: UserDefaultsKeys.accountToken.rawValue)
        }
    }

    var formattedToken: String? {
        return token?.split(every: 4).joined(separator: " ")
    }

    /// Returns the account expiry for the currently used account token
    private(set) var expiry: Date? {
        set {
            UserDefaults.standard.set(newValue, forKey: UserDefaultsKeys.accountExpiry.rawValue)
        }
        get {
            return UserDefaults.standard.object(forKey: UserDefaultsKeys.accountExpiry.rawValue) as? Date
        }
    }

    var isLoggedIn: Bool {
        return token != nil
    }

    /// Save the boolean flag in preferences indicating that the user agreed to terms of service.
    func agreeToTermsOfService() {
        UserDefaults.standard.set(true, forKey: UserDefaultsKeys.isAgreedToTermsOfService.rawValue)
    }

    func loginWithNewAccount(completionHandler: @escaping (Result<REST.AccountResponse, Account.Error>) -> Void) {
        let operation = AsyncBlockOperation { operation in
            _ = REST.Client.shared.createAccount().execute { result in
                DispatchQueue.main.async {
                    switch result {
                    case .success(let response):
                        self.setupTunnel(accountToken: response.token, expiry: response.expires) { error in
                            if let error = error {
                                completionHandler(.failure(error))
                            } else {
                                self.observerList.forEach { observer in
                                    observer.account(self, didLoginWithToken: response.token, expiry: response.expires)
                                }

                                completionHandler(.success(response))
                            }

                            operation.finish()
                        }

                    case .failure(let error):
                        completionHandler(.failure(.createAccount(error)))

                        operation.finish()
                    }
                }
            }
        }

        operationQueue.addOperation(operation)
    }

    /// Perform the login and save the account token along with expiry (if available) to the
    /// application preferences.
    func login(accountToken: String, completionHandler: @escaping (Result<REST.AccountResponse, Account.Error>) -> Void) {
        let operation = AsyncBlockOperation { operation in
            _ = REST.Client.shared.getAccountExpiry(token: accountToken)
                .execute(retryStrategy: .default) { result in
                    DispatchQueue.main.async {
                        switch result {
                        case .success(let response):
                            self.setupTunnel(accountToken: response.token, expiry: response.expires) { error in
                                if let error = error {
                                    completionHandler(.failure(error))
                                } else {
                                    self.observerList.forEach { observer in
                                        observer.account(self, didLoginWithToken: response.token, expiry: response.expires)
                                    }
                                    completionHandler(.success(response))
                                }
                                operation.finish()
                            }

                        case .failure(let error):
                            completionHandler(.failure(.verifyAccount(error)))
                            operation.finish()
                        }
                    }
                }
        }

        operationQueue.addOperation(operation)
    }

    /// Perform the logout by erasing the account token and expiry from the application preferences.
    func logout(completionHandler: @escaping () -> Void) {
        let operation = AsyncBlockOperation { operation in
            TunnelManager.shared.unsetAccount {
                self.removeFromPreferences()
                self.observerList.forEach { (observer) in
                    observer.accountDidLogout(self)
                }

                completionHandler()
                operation.finish()
            }
        }

        operationQueue.addOperation(operation)
    }

    /// Forget that user was logged in, but do not attempt to unset account in `TunnelManager`.
    /// This function is used in cases where the tunnel or tunnel settings are corrupt.
    func forget(completionHandler: @escaping () -> Void) {
        let operation = AsyncBlockOperation { operation in
            DispatchQueue.main.async {
                self.removeFromPreferences()
                self.observerList.forEach { (observer) in
                    observer.accountDidLogout(self)
                }
                operation.finish()
            }
        }

        operationQueue.addOperation(operation)
    }

    func updateAccountExpiry() {
        let operation = AsyncBlockOperation { operation in
            DispatchQueue.main.async {
                guard let token = self.token else {
                    operation.finish()
                    return
                }

                _ = REST.Client.shared.getAccountExpiry(token: token)
                    .execute(retryStrategy: .default) { result in
                        switch result {
                        case .success(let response):
                            if self.expiry != response.expires {
                                self.expiry = response.expires
                                self.observerList.forEach { (observer) in
                                    observer.account(self, didUpdateExpiry: response.expires)
                                }
                            }

                        case .failure(let error):
                            self.logger.error(chainedError: error, message: "Failed to update account expiry.")
                        }

                        operation.finish()
                    }
            }
        }

        operationQueue.addOperation(operation)
    }

    private func setupTunnel(accountToken: String, expiry: Date, completionHandler: @escaping (Account.Error?) -> Void) {
        TunnelManager.shared.setAccount(accountToken: accountToken) { error in
            dispatchPrecondition(condition: .onQueue(.main))

            if let error = error {
                completionHandler(.tunnelConfiguration(error))
            } else {
                self.token = accountToken
                self.expiry = expiry

                completionHandler(nil)
            }
        }
    }

    private func removeFromPreferences() {
        let preferences = UserDefaults.standard

        preferences.removeObject(forKey: UserDefaultsKeys.accountToken.rawValue)
        preferences.removeObject(forKey: UserDefaultsKeys.accountExpiry.rawValue)
    }

    // MARK: - Account observation

    func addObserver(_ observer: AccountObserver) {
        observerList.append(observer)
    }

    func removeObserver(_ observer: AccountObserver) {
        observerList.remove(observer)
    }
}

extension Account: AppStorePaymentObserver {

    func startPaymentMonitoring(with paymentManager: AppStorePaymentManager) {
        paymentManager.addPaymentObserver(self)
    }

    func appStorePaymentManager(_ manager: AppStorePaymentManager, transaction: SKPaymentTransaction, accountToken: String?, didFailWithError error: AppStorePaymentManager.Error) {
        // no-op
    }

    func appStorePaymentManager(_ manager: AppStorePaymentManager, transaction: SKPaymentTransaction, accountToken: String, didFinishWithResponse response: REST.CreateApplePaymentResponse) {
        let operation = AsyncBlockOperation { operation in
            DispatchQueue.main.async {
                let newExpiry = response.newExpiry

                // Make sure that payment corresponds to the active account token
                if self.token == accountToken, self.expiry != newExpiry {
                    self.expiry = newExpiry
                    self.observerList.forEach { (observer) in
                        observer.account(self, didUpdateExpiry: newExpiry)
                    }
                }

                operation.finish()
            }
        }

        operationQueue.addOperation(operation)
    }
}
