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
typealias AppResetManager = AppResetManagerNoOp
#else
typealias AppResetManager = AppResetManagerNoOp
#endif

@MainActor
final class AppResetManagerReal {
    private let launchArguments: LaunchArguments
    private let tunnelManager: TunnelManager
    private let settingsManager: SettingsManager
    private var tunnelObserver: TunnelObserver!

    let logger = Logger(label: "AppResetManager")

    var onAppReady: (@Sendable @MainActor () -> Void)?

    private let isConfigurationLoaded = CurrentValueSubject<Bool, Never>(false)
    private let isAppReady = CurrentValueSubject<Bool, Never>(false)

    private var cancellables = Set<AnyCancellable>()

    init(
        launchArguments: LaunchArguments,
        tunnelManager: TunnelManager,
        settingsManager: SettingsManager
    ) {
        self.launchArguments = launchArguments
        self.tunnelManager = tunnelManager
        self.settingsManager = settingsManager
    }

    func start() {
        observeReadiness()
        addObserver()

        // Outside of UI tests there is no reset flow to wait for, so the app is
        // considered ready as soon as the tunnel configuration has loaded.
        guard launchArguments.target.isUITest else {
            isAppReady.send(true)
            return
        }

        Task {
            async let cleanup: () = StorePaymentManager.cleanupUnfinishedTransactions()
            async let setupTask: () = setup()
            _ = await (cleanup, setupTask)
        }
    }

    private func observeReadiness() {
        Publishers.CombineLatest(isConfigurationLoaded, isAppReady)
            .filter { configurationLoaded, appReady in
                configurationLoaded && appReady
            }
            .sink { [weak self] _, _ in
                guard let self else { return }
                tunnelManager.removeObserver(tunnelObserver)
                onAppReady?()
            }
            .store(in: &cancellables)
    }

    private func addObserver() {
        let tunnelObserver = TunnelBlockObserver(
            didLoadConfiguration: { [weak self] tunnelManager in
                self?.isConfigurationLoaded.send(true)
            },
            didUpdateTunnelStatus: { [weak self] tunnelManager, tunnelStatus in
                guard let self, launchArguments.target.isUITest else { return }
                if tunnelStatus.observedState != .disconnected {
                    tunnelManager.stopTunnel()
                } else if case .disconnected = tunnelStatus.observedState {
                    Task {
                        await reset()
                    }
                }
            }
        )
        tunnelManager.addObserver(tunnelObserver)
        self.tunnelObserver = tunnelObserver

        // If configuration has already finished loading, let the readiness check catch up manually.
        if tunnelManager.isConfigurationLoaded {
            isConfigurationLoaded.send(true)
        }
    }

    private func setup() async {
        do {
            guard try await isTunnelActive() == false else {
                // Don't just wait for a status change that may never come if the tunnel is
                // already sitting in a stable connected/connecting state - actively stop it so
                // `didUpdateTunnelStatus` eventually reports `.disconnected` and triggers `reset()`.
                tunnelManager.stopTunnel()
                return
            }
            await reset()
        } catch {
            logger.error("Unexpected tunnel error: \(error.localizedDescription)")
            isAppReady.send(true)
        }
    }

    private func reset() async {
        switch tunnelManager.deviceState {
        case .loggedIn:
            await logoutIfNeeded()
            fallthrough
        default:
            resetUserDefaults()
            await resetKeychain()
            isAppReady.send(true)
        }
    }

    private func logoutIfNeeded() async {
        guard launchArguments.authenticationState == .forceLoggedOut else {
            return
        }
        await tunnelManager.unsetAccount(isRemovingProfile: false)
    }

    private func resetKeychain() async {
        let policy = launchArguments.settingsResetPolicy
        settingsManager.resetStore(policy: policy.toSettingsResetPolicy)
        if policy.shouldReset(.settings) {
            await tunnelManager.updateSettings([.reset])
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

    private func isTunnelActive() async throws -> Bool {
        #if targetEnvironment(simulator)
            return false
        #else
            try await withCheckedThrowingContinuation { continuation in
                NETunnelProviderManager.loadAllFromPreferences { managers, error in
                    if let error {
                        continuation.resume(throwing: error)
                        return
                    }

                    let active = (managers ?? []).contains {
                        [.connected, .connecting, .reasserting].contains($0.connection.status)
                    }

                    continuation.resume(returning: active)
                }
            }
        #endif
    }
}

@MainActor
final class AppResetManagerNoOp {
    private let tunnelManager: TunnelManager
    private var tunnelObserver: TunnelObserver!

    var onAppReady: (@Sendable @MainActor () -> Void)?

    init(
        launchArguments: LaunchArguments,
        tunnelManager: TunnelManager,
        settingsManager: SettingsManager
    ) {
        self.tunnelManager = tunnelManager
    }

    func start() {
        let tunnelObserver = TunnelBlockObserver(
            didLoadConfiguration: { [weak self] _ in
                self?.onAppReady?()
            }
        )
        tunnelManager.addObserver(tunnelObserver)
        self.tunnelObserver = tunnelObserver

        // If configuration has already finished loading, let UI catch up manually.
        if tunnelManager.isConfigurationLoaded {
            onAppReady?()
        }
    }
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
