//
//  Account.swift
//  MullvadVPN
//
//  Created by pronebird on 16/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Combine
import Foundation
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

/// A enum holding the `UserDefaults` string keys
private enum UserDefaultsKeys: String {
    case accountToken = "accountToken"
    case accountExpiry = "accountExpiry"
}

/// A class that groups the account related operations
class Account {

    static let shared = Account()
    private let apiClient = MullvadAPI()

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

