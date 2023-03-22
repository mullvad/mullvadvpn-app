//
//  LoginInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 27/01/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging

final class LoginInteractor {
    private let tunnelManager: TunnelManager
    private let logger = Logger(label: "LoginInteractor")

    init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager
    }

    func setAccount(accountNumber: String, completion: @escaping (Error?) -> Void) {
        tunnelManager.setExistingAccount(accountNumber: accountNumber) { result in
            completion(result.error)
        }
    }

    func createAccount(completion: @escaping (Result<String, Error>) -> Void) {
        tunnelManager.setNewAccount { result in
            completion(result.map { $0.number })
        }
    }

    func getLastUsedAccount() -> String? {
        do {
            return try SettingsManager.getLastUsedAccount()
        } catch {
            logger.error(
                error: error,
                message: "Failed to get last used account."
            )
            return nil
        }
    }

    func removeLastUsedAccount() {
        do {
            try SettingsManager.setLastUsedAccount(nil)
        } catch {
            logger.error(
                error: error,
                message: "Failed to remove last used account."
            )
        }
    }
}
