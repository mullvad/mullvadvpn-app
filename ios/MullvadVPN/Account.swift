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

/// A type-erasing weak container for `AccountObserver`
private class AnyAccountObserver: AccountObserver, WeakObserverBox, Equatable {
    private(set) weak var inner: AccountObserver?

    init<T: AccountObserver>(_ inner: T) {
        self.inner = inner
    }

    func account(_ account: Account, didUpdateExpiry expiry: Date) {
        inner?.account(account, didUpdateExpiry: expiry)
    }

    func account(_ account: Account, didLoginWithToken token: String, expiry: Date) {
        inner?.account(account, didLoginWithToken: token, expiry: expiry)
    }

    func accountDidLogout(_ account: Account) {
        inner?.accountDidLogout(account)
    }

    static func == (lhs: AnyAccountObserver, rhs: AnyAccountObserver) -> Bool {
        return lhs.inner === rhs.inner
    }
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
    }

    /// A shared instance of `Account`
    static let shared = Account()

    private let logger = Logger(label: "Account")
    private var observerList = ObserverList<AnyAccountObserver>()

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

    private let dispatchQueue = DispatchQueue(label: "AccountQueue")

    var isLoggedIn: Bool {
        return token != nil
    }

    /// Save the boolean flag in preferences indicating that the user agreed to terms of service.
    func agreeToTermsOfService() {
        UserDefaults.standard.set(true, forKey: UserDefaultsKeys.isAgreedToTermsOfService.rawValue)
    }

    func loginWithNewAccount() -> Result<REST.AccountResponse, Account.Error>.Promise {
        return REST.Client.shared.createAccount()
            .execute()
            .mapError { error in
                return Error.createAccount(error)
            }
            .receive(on: .main)
            .mapThen { response in
                return self.setupTunnel(accountToken: response.token, expiry: response.expires)
                    .map { _ in
                        self.observerList.forEach { (observer) in
                            observer.account(self, didLoginWithToken: response.token, expiry: response.expires)
                        }
                        return response
                    }
            }
            .block(on: dispatchQueue)
            .receive(on: .main)
    }

    /// Perform the login and save the account token along with expiry (if available) to the
    /// application preferences.
    func login(with accountToken: String) -> Result<REST.AccountResponse, Account.Error>.Promise {
        return REST.Client.shared.getAccountExpiry(token: accountToken)
            .execute(retryStrategy: .default)
            .mapError { error in
                return Account.Error.verifyAccount(error)
            }
            .mapThen { response in
                return self.setupTunnel(accountToken: response.token, expiry: response.expires)
                    .map { _ in
                        self.observerList.forEach { (observer) in
                            observer.account(self, didLoginWithToken: response.token, expiry: response.expires)
                        }
                        return response
                    }
            }
            .block(on: dispatchQueue)
            .receive(on: .main)
    }

    /// Perform the logout by erasing the account token and expiry from the application preferences.
    func logout() -> Result<(), Account.Error>.Promise {
        return TunnelManager.shared.unsetAccount()
            .mapError { error in
                return Account.Error.tunnelConfiguration(error)
            }
            .receive(on: .main)
            .onSuccess { _ in
                self.removeFromPreferences()
                self.observerList.forEach { (observer) in
                    observer.accountDidLogout(self)
                }
            }
            .block(on: dispatchQueue)
            .receive(on: .main)
    }

    /// Forget that user was logged in, but do not attempt to unset account in `TunnelManager`.
    /// This function is used in cases where the tunnel or tunnel settings are corrupt.
    func forget() -> Promise<Void> {
        return Promise<Void> { resolver in
            self.removeFromPreferences()
            self.observerList.forEach { (observer) in
                observer.accountDidLogout(self)
            }
            resolver.resolve(value: ())
        }
        .schedule(on: .main)
        .block(on: dispatchQueue)
        .receive(on: .main)
    }

    func updateAccountExpiry() {
        Promise<String?>.deferred { self.token }
            .mapThen(defaultValue: nil) { token in
                return REST.Client.shared.getAccountExpiry(token: token)
                    .execute(retryStrategy: .default)
                    .onFailure { error in
                        self.logger.error(chainedError: error, message: "Failed to update account expiry")
                    }
                    .success()
            }
            .schedule(on: .main)
            .block(on: dispatchQueue)
            .receive(on: .main)
            .observe { completion in
                guard let response = completion.flattenUnwrappedValue else { return }

                if self.expiry != response.expires {
                    self.expiry = response.expires
                    self.observerList.forEach { (observer) in
                        observer.account(self, didUpdateExpiry: response.expires)
                    }
                }
            }
    }

    private func setupTunnel(accountToken: String, expiry: Date) -> Result<(), Error>.Promise {
        return TunnelManager.shared.setAccount(accountToken: accountToken)
            .receive(on: .main)
            .mapError { error in
                return Error.tunnelConfiguration(error)
            }
            .onSuccess { _ in
                self.token = accountToken
                self.expiry = expiry
            }
    }

    private func removeFromPreferences() {
        let preferences = UserDefaults.standard

        preferences.removeObject(forKey: UserDefaultsKeys.accountToken.rawValue)
        preferences.removeObject(forKey: UserDefaultsKeys.accountExpiry.rawValue)
    }

    // MARK: - Account observation

    func addObserver<T: AccountObserver>(_ observer: T) {
        observerList.append(AnyAccountObserver(observer))
    }

    func removeObserver<T: AccountObserver>(_ observer: T) {
        observerList.remove(AnyAccountObserver(observer))
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
        dispatchQueue.async {
            DispatchQueue.main.sync {
                let newExpiry = response.newExpiry

                // Make sure that payment corresponds to the active account token
                if self.token == accountToken, self.expiry != newExpiry {
                    self.expiry = newExpiry
                    self.observerList.forEach { (observer) in
                        observer.account(self, didUpdateExpiry: newExpiry)
                    }
                }
            }
        }
    }
}
