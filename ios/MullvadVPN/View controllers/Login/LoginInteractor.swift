//
//  LoginInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 27/01/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
@preconcurrency import MullvadLogging
import MullvadSettings

final class LoginInteractor: @unchecked Sendable {
    private let tunnelManager: TunnelManager
    private let logger = Logger(label: "LoginInteractor")
    private var tunnelObserver: TunnelObserver?
    var didCreateAccount: (@MainActor @Sendable () -> Void)?
    var suggestPreferredAccountNumber: (@Sendable (String) -> Void)?

    init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager
    }

    func setAccount(accountNumber: String) async throws {
        _ = try await tunnelManager.setExistingAccount(accountNumber: accountNumber)
    }

    func createAccount() async throws -> String {
        let accountNumber = try await tunnelManager.setNewAccount().number
        await didCreateAccount?()

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
