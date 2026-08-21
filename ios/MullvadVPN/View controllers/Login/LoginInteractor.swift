// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

@preconcurrency import MullvadLogging
import MullvadSettings

final class LoginInteractor: @unchecked Sendable {
    private let tunnelManager: LoginViewModelProviding
    private let logger = Logger(label: "LoginInteractor")
    private var tunnelObserver: TunnelObserver?
    private let settingsManager: SettingsManager

    var didCreateAccount: (@MainActor @Sendable () -> Void)?

    var hasLastAccountNumber: Bool {
        getLastUsedAccount() != nil
    }

    init(tunnelManager: LoginViewModelProviding, settingsManager: SettingsManager) {
        self.tunnelManager = tunnelManager
        self.settingsManager = settingsManager
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
            return try settingsManager.getLastUsedAccount()
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
            try settingsManager.setLastUsedAccount(nil)
        } catch {
            logger.error(
                error: error,
                message: "Failed to remove last used account."
            )
        }
    }
}
