//
//  LoginInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 27/01/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadSettings

final class LoginInteractor {
    private let tunnelManager: TunnelManager
    private let logger = Logger(label: "LoginInteractor")
    private var tunnelObserver: TunnelObserver?
    var didCreateAccount: (() -> Void)?
    var suggestPreferredAccountNumber: ((String) -> Void)?

    init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager
    }

    func setAccount(accountNumber: String) async throws {
        _ = try await tunnelManager.setExistingAccount(accountNumber: accountNumber)
    }

    func createAccount() async throws -> String {
        let accountNumber = try await tunnelManager.setNewAccount().number
        didCreateAccount?()

        return accountNumber
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
