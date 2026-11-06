// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Combine
import MullvadLogging
import MullvadSettings
import NetworkExtension
import UIKit

#if NEVER_IN_PRODUCTION
    typealias AppResetManager = AppResetManagerReal
#else
    typealias AppResetManager = AppResetManagerNoOp
#endif

@MainActor
final class AppResetManagerReal {
    private let launchArguments: LaunchArguments
    private let settingsManager: SettingsManager
    let logger = Logger(label: "AppResetManager")

    init(
        launchArguments: LaunchArguments,
        settingsManager: SettingsManager
    ) {
        self.launchArguments = launchArguments
        self.settingsManager = settingsManager
    }

    func start() async {
        guard launchArguments.target.isUITest else { return }
        await StorePaymentManager.cleanupUnfinishedTransactions()
        await reset()
    }

    private func reset() async {
        await logoutIfNeeded()
        resetUserDefaults()
        await resetKeychain()
    }

    private func logoutIfNeeded() async {
        do {
            try await stopActiveTunnel()
            guard launchArguments.authenticationState == .forceLoggedOut else {
                return
            }
            try settingsManager.writeDeviceState(.loggedOut)
            try settingsManager.setLastUsedAccount(nil)
        } catch {
            logger.error("Could not log out properly: \(error)")
        }
    }

    private func resetKeychain() async {
        let policy = launchArguments.settingsResetPolicy
        settingsManager.resetStore(policy: policy.toSettingsResetPolicy)
        if policy.shouldReset(.settings) {
            try? settingsManager.writeSettings(LatestTunnelSettings())
        }
    }

    private func resetUserDefaults() {
        let policy = launchArguments.appPreferencesResetPolicy
        let defaults = UserDefaults.standard
        let keysToRemove: Set<UITestAppPreferencesKey> = policy.resolvedKeys()
        for key in keysToRemove {
            defaults.removeObject(forKey: key.rawValue)
        }
        defaults.synchronize()
    }

    private func stopActiveTunnel() async throws {
        NETunnelProviderManager.loadAllFromPreferences { managers, _ in
            _ = managers?.compactMap { manager in
                manager.isOnDemandEnabled = false
                manager.connection.stopVPNTunnel()
                manager.saveToPreferences()
            }
        }
    }
}

@MainActor
final class AppResetManagerNoOp {
    init(
        launchArguments: LaunchArguments,
        settingsManager: SettingsManager
    ) {}

    func start() {}
}

extension UITestSettingsKey {
    var toDomain: SettingsKey {
        switch self {
        case .settings: return .settings
        case .ipOverrides: return .ipOverrides
        case .customRelayLists: return .customRelayLists
        case .recentConnections: return .recentConnections
        }
    }
}

private extension UITestSettingsResetPolicy {
    var toSettingsResetPolicy: SettingsResetPolicy {
        switch self {
        case .none:
            .none
        case .allExcept(let keys):
            .allExcept(Set(keys.map(\.toDomain)))
        case .only(let keys):
            .only(Set(keys.map(\.toDomain)))
        case .all:
            .all
        }
    }
}
