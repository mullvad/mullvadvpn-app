//
//  AppResetManager.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-02-23.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadLogging
import MullvadSettings
import NetworkExtension
import UIKit

@MainActor
final class AppResetManager {
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
        guard launchArguments.target.isUITest else { return }
        observeReadiness()
        addObserver()
    }

    func start() {
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
                guard let self else { return }
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
    }

    private func setup() async {
        do {
            guard try await isTunnelActive() == false else { return }
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
            resetKeychain()
            isAppReady.send(true)
        }
    }

    private func logoutIfNeeded() async {
        guard launchArguments.authenticationState == .forceLoggedOut else {
            return
        }
        await tunnelManager.unsetAccount(isRemovingProfile: false)
    }

    private func resetKeychain() {
        let policy = launchArguments.settingsResetPolicy
        settingsManager.resetStore(policy: policy.toSettingsResetPolicy)
        if policy.shouldReset(.settings) {
            tunnelManager.updateSettings([.reset])
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
