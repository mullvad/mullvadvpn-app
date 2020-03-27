//
//  Account.swift
//  MullvadVPN
//
//  Created by pronebird on 16/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation
import NetworkExtension
import StoreKit
import os

/// A enum describing the errors emitted by `Account`
enum AccountError: Error {
    /// A failure to perform the login
    case login(AccountLoginError)

    /// A failure to log out
    case logout(TunnelManagerError)
}

/// A enum describing the error emitted during login
enum AccountLoginError: Error {
    case invalidAccount
    case tunnelConfiguration(TunnelManagerError)
}

extension AccountError: LocalizedError {
    var errorDescription: String? {
        switch self {
        case .login:
            return NSLocalizedString("Log in error", comment: "")

        case .logout:
            return NSLocalizedString("Log out error", comment: "")
        }
    }

    var failureReason: String? {
        switch self {
        case .login(.invalidAccount):
            return NSLocalizedString("Invalid account", comment: "")

        case .login(.tunnelConfiguration(.setAccount(let setAccountError))):
            switch setAccountError {
            case .pushWireguardKey(.transport(.network)):
                return NSLocalizedString("Network error", comment: "")

            case .pushWireguardKey(.server(let serverError)):
                return serverError.errorDescription ?? serverError.message

            case .setup(.saveTunnel(let systemError as NEVPNError))
                where systemError.code == .configurationReadWriteFailed:
                return NSLocalizedString("Permission denied to add a VPN profile", comment: "")

            default:
                return NSLocalizedString("Internal error", comment: "")
            }

        case .logout:
            return NSLocalizedString("Internal error", comment: "")

        default:
            return nil
        }
    }
}

/// A enum holding the `UserDefaults` string keys
private enum UserDefaultsKeys: String {
    case isAgreedToTermsOfService = "isAgreedToTermsOfService"
    case accountToken = "accountToken"
    case accountExpiry = "accountExpiry"
}

/// A class that groups the account related operations
class Account {

    /// A notification name used to broadcast the changes to account expiry
    static let didUpdateAccountExpiryNotification = Notification.Name("didUpdateAccountExpiry")

    /// A notification userInfo key that holds the `Date` with the new account expiry
    static let newAccountExpiryUserInfoKey = "newAccountExpiry"

    static let shared = Account()
    private let apiClient = MullvadAPI()

    /// Returns true if user agreed to terms of service, otherwise false
    var isAgreedToTermsOfService: Bool {
        return UserDefaults.standard.bool(forKey: UserDefaultsKeys.isAgreedToTermsOfService.rawValue)
    }

    /// Returns the currently used account token
    var token: String? {
        return UserDefaults.standard.string(forKey: UserDefaultsKeys.accountToken.rawValue)
    }

    /// Returns the account expiry for the currently used account token
    var expiry: Date? {
        return UserDefaults.standard.object(forKey: UserDefaultsKeys.accountExpiry.rawValue) as? Date
    }

    var isLoggedIn: Bool {
        return token != nil
    }

    /// Save the boolean flag in preferences indicating that the user agreed to terms of service.
    func agreeToTermsOfService() {
        UserDefaults.standard.set(true, forKey: UserDefaultsKeys.isAgreedToTermsOfService.rawValue)
    }

    /// Perform the login and save the account token along with expiry (if available) to the
    /// application preferences.
    func login(with accountToken: String) -> AnyPublisher<(), AccountError> {
        return apiClient.verifyAccount(accountToken: accountToken)
            .setFailureType(to: AccountLoginError.self)
            .handleEvents(receiveOutput: { (accountVerification) in
                if case .deferred(let error) = accountVerification {
                    os_log(.error, "Failed to verify the account: %{public}s", error.localizedDescription)
                }
            })
            .flatMap {
                self.handleVerification($0).publisher
                    .flatMap { (expiry) in
                        TunnelManager.shared.setAccount(accountToken: accountToken)
                            .mapError { AccountLoginError.tunnelConfiguration($0) }
                            .map { expiry }
                }
        }.mapError { AccountError.login($0) }
            .receive(on: DispatchQueue.main).map { (expiry) in
                self.saveAccountToPreferences(accountToken: accountToken, expiry: expiry)
        }.eraseToAnyPublisher()
    }

    /// Perform the logout by erasing the account token and expiry from the application preferences.
    func logout() -> AnyPublisher<(), AccountError> {
        return TunnelManager.shared.unsetAccount()
            .receive(on: DispatchQueue.main)
            .mapError { AccountError.logout($0) }
            .map(self.removeAccountFromPreferences)
            .eraseToAnyPublisher()
    }

    private func handleVerification(_ verification: AccountVerification) -> Result<Date?, AccountLoginError> {
        switch verification {
        case .deferred:
            return .success(nil)
        case .verified(let expiry):
            return .success(expiry)
        case .invalid:
            return .failure(.invalidAccount)
        }
    }

    private func saveAccountToPreferences(accountToken: String, expiry: Date?) {
        let preferences = UserDefaults.standard

        preferences.set(accountToken, forKey: UserDefaultsKeys.accountToken.rawValue)
        preferences.set(expiry, forKey: UserDefaultsKeys.accountExpiry.rawValue)
    }

    private func removeAccountFromPreferences() {
        let preferences = UserDefaults.standard

        preferences.removeObject(forKey: UserDefaultsKeys.accountToken.rawValue)
        preferences.removeObject(forKey: UserDefaultsKeys.accountExpiry.rawValue)

    }
}

extension Account: AppStorePaymentObserver {

    func startPaymentMonitoring(with paymentManager: AppStorePaymentManager) {
        paymentManager.addPaymentObserver(self)
    }

    func appStorePaymentManager(_ manager: AppStorePaymentManager, transaction: SKPaymentTransaction, didFailWithError error: AppStorePaymentManager.Error) {
        // no-op
    }

    func appStorePaymentManager(_ manager: AppStorePaymentManager, transaction: SKPaymentTransaction, didFinishWithResponse response: SendAppStoreReceiptResponse) {
        UserDefaults.standard.set(response.newExpiry,
                                  forKey: UserDefaultsKeys.accountExpiry.rawValue)

        NotificationCenter.default.post(
            name: Self.didUpdateAccountExpiryNotification,
            object: self, userInfo: [Self.newAccountExpiryUserInfoKey: response.newExpiry]
        )
    }
}
