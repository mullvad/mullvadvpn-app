//
//  Account.swift
//  MullvadVPN
//
//  Created by pronebird on 16/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension
import StoreKit
import os

/// A enum holding the `UserDefaults` string keys
private enum UserDefaultsKeys: String {
    case isAgreedToTermsOfService = "isAgreedToTermsOfService"
    case accountToken = "accountToken"
    case accountExpiry = "accountExpiry"
}

/// A class that groups the account related operations
class Account {

    enum Error: ChainedError {
        /// A failure to create the new account token
        case createAccount(MullvadRpc.Error)

        /// A failure to verify the account token
        case verifyAccount(MullvadRpc.Error)

        /// A failure to configure a tunnel
        case tunnelConfiguration(TunnelManager.Error)
    }

    /// A notification name used to broadcast the changes to account expiry
    static let didUpdateAccountExpiryNotification = Notification.Name("didUpdateAccountExpiry")

    /// A notification userInfo key that holds the `Date` with the new account expiry
    static let newAccountExpiryUserInfoKey = "newAccountExpiry"

    /// A shared instance of `Account`
    static let shared = Account()

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

    private enum ExclusivityCategory {
        case exclusive
    }

    private let rpc = MullvadRpc.withEphemeralURLSession()
    private let operationQueue = OperationQueue()
    private lazy var exclusivityController = ExclusivityController<ExclusivityCategory>(operationQueue: operationQueue)

    var isLoggedIn: Bool {
        return token != nil
    }

    /// Save the boolean flag in preferences indicating that the user agreed to terms of service.
    func agreeToTermsOfService() {
        UserDefaults.standard.set(true, forKey: UserDefaultsKeys.isAgreedToTermsOfService.rawValue)
    }

    func loginWithNewAccount(completionHandler: @escaping (Result<(String, Date), Error>) -> Void) {
        let operation = rpc.createAccount().operation()

        operation.addDidFinishBlockObserver({ (operation, result) in
            DispatchQueue.main.async {
                switch result {
                case .success(let newAccountToken):
                    let expiry = Date()
                    self.setupTunnel(accountToken: newAccountToken, expiry: expiry) { (result) in
                        completionHandler(result.map { (newAccountToken, expiry) })
                    }

                case .failure(let error):
                    completionHandler(.failure(.createAccount(error)))
                }
            }
        })

        exclusivityController.addOperation(operation, categories: [.exclusive])
    }

    /// Perform the login and save the account token along with expiry (if available) to the
    /// application preferences.
    func login(with accountToken: String, completionHandler: @escaping (Result<Date, Error>) -> Void) {
        let operation = rpc.getAccountExpiry(accountToken: accountToken)
            .operation()

        operation.addDidFinishBlockObserver { (operation, result) in
            DispatchQueue.main.async {
                switch result {
                case .success(let expiry):
                    self.setupTunnel(accountToken: accountToken, expiry: expiry) { (result) in
                        completionHandler(result.map { expiry })
                    }

                case .failure(let error):
                    completionHandler(.failure(.verifyAccount(error)))
                }
            }
        }

        exclusivityController.addOperation(operation, categories: [.exclusive])
    }

    /// Perform the logout by erasing the account token and expiry from the application preferences.
    func logout(completionHandler: @escaping (Result<(), Error>) -> Void) {
        let operation = ResultOperation<(), Error> { (finish) in
            TunnelManager.shared.unsetAccount { (result) in
                DispatchQueue.main.async {
                    switch result {
                    case .success:
                        self.removeFromPreferences()

                        finish(.success(()))

                    case .failure(let error):
                        finish(.failure(.tunnelConfiguration(error)))
                    }
                }
            }
        }

        operation.addDidFinishBlockObserver { (operation, result) in
            DispatchQueue.main.async {
                completionHandler(result)
            }
        }

        exclusivityController.addOperation(operation, categories: [.exclusive])
    }

    func updateAccountExpiry() {
        let makeRequest = ResultOperation { () -> MullvadRpc.Request<Date>? in
            return self.token.flatMap { (accountToken) -> MullvadRpc.Request<Date>? in
                self.rpc.getAccountExpiry(accountToken: accountToken)
            }
        }

        let sendRequest = rpc.getAccountExpiry()
            .injectResult(from: makeRequest)

        sendRequest.addDidFinishBlockObserver { (operation, result) in
            DispatchQueue.main.async {
                switch result {
                case .success(let expiry):
                    self.expiry = expiry
                    self.postExpiryUpdateNotification(newExpiry: expiry)

                case .failure(let error):
                    error.logChain(message: "Failed to update account expiry")
                }
            }
        }

        exclusivityController.addOperations([makeRequest, sendRequest], categories: [.exclusive])
    }

    private func setupTunnel(accountToken: String, expiry: Date, completionHandler: @escaping (Result<(), Error>) -> Void) {
        TunnelManager.shared.setAccount(accountToken: accountToken) { (managerResult) in
            DispatchQueue.main.async {
                switch managerResult {
                case .success:
                    self.token = accountToken
                    self.expiry = expiry

                    completionHandler(.success(()))

                case .failure(let error):
                    completionHandler(.failure(.tunnelConfiguration(error)))
                }
            }
        }
    }

    private func removeFromPreferences() {
        let preferences = UserDefaults.standard

        preferences.removeObject(forKey: UserDefaultsKeys.accountToken.rawValue)
        preferences.removeObject(forKey: UserDefaultsKeys.accountExpiry.rawValue)

    }

    fileprivate func postExpiryUpdateNotification(newExpiry: Date) {
        NotificationCenter.default.post(
            name: Self.didUpdateAccountExpiryNotification,
            object: self, userInfo: [Self.newAccountExpiryUserInfoKey: newExpiry]
        )
    }
}

extension Account: AppStorePaymentObserver {

    func startPaymentMonitoring(with paymentManager: AppStorePaymentManager) {
        paymentManager.addPaymentObserver(self)
    }

    func appStorePaymentManager(_ manager: AppStorePaymentManager, transaction: SKPaymentTransaction, accountToken: String?, didFailWithError error: AppStorePaymentManager.Error) {
        // no-op
    }

    func appStorePaymentManager(_ manager: AppStorePaymentManager, transaction: SKPaymentTransaction, accountToken: String, didFinishWithResponse response: SendAppStoreReceiptResponse) {
        let newExpiry = response.newExpiry

        let operation = AsyncBlockOperation { (finish) in
            DispatchQueue.main.async {
                // Make sure that payment corresponds to the active account token
                if self.token == accountToken {
                    self.expiry = newExpiry
                    self.postExpiryUpdateNotification(newExpiry: newExpiry)
                }

                finish()
            }
        }

        exclusivityController.addOperation(operation, categories: [.exclusive])
    }
}
