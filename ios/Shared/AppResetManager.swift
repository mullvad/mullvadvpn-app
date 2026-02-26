//
//  AppResetManager.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-02-23.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadSettings
import NetworkExtension
import UIKit

@MainActor
final class AppResetManager {
    private let launchArguments: LaunchArguments
    private let tunnelManager: TunnelManager
    private var tunnelObserver: TunnelObserver!
    let logger = Logger(label: "AppResetManager")

    var onAppReady: (@Sendable @MainActor () -> Void)?

    init(
        launchArguments: LaunchArguments,
        tunnelManager: TunnelManager
    ) {
        self.launchArguments = launchArguments
        self.tunnelManager = tunnelManager
        guard launchArguments.target == .uiTests else { return }
        addObserver()
        Task {
            await setup()
        }
    }

    private func addObserver() {
        let tunnelObserver = TunnelBlockObserver(
            didUpdateDeviceState: {
                [weak self] tunnelManager, deviceState, previousDeviceState in
                guard let self else { return }
                if case .revoked = deviceState, launchArguments.target.isUITest {
                    resetKeychain()
                    resetUserDefaults()
                }
            })
        tunnelManager.addObserver(tunnelObserver)
        self.tunnelObserver = tunnelObserver
    }

    private func setup() async {
        do {
            try await disableVPN()
            await reset()
        } catch {
            logger.error("Unexpected tunnel error: \(error.localizedDescription)")
            onAppReady?()
        }
    }

    private func reset() async {
        switch tunnelManager.deviceState {
        case .revoked:
            resetUserDefaults()
            resetKeychain()
            onAppReady?()
        case .loggedIn:
            await logoutIfNeeded()
            fallthrough
        default:
            resetStorageIfNeeded()
            onAppReady?()
        }
    }

    private func logoutIfNeeded() async {
        if launchArguments.authenticationState == .forceLoggedOut {
            await tunnelManager.unsetAccount(isRemovingProfile: false)
        }
    }

    private func resetStorageIfNeeded() {
        switch launchArguments.localDataResetPolicy {
        case .none:
            break
        case .all:
            resetUserDefaults()
            resetKeychain()
        }
    }

    private func resetKeychain() {
        SettingsManager.resetStore(policy: .all)
        tunnelManager.updateSettings([.reset])
    }

    private func resetUserDefaults() {
        guard let bundleID = Bundle.main.bundleIdentifier else { return }
        UserDefaults.standard.removePersistentDomain(forName: bundleID)
    }

    private func disableVPN() async throws {
        let managers = try await NETunnelProviderManager.loadAllFromPreferences()
        let manager = managers.first {
            $0.connection.status == .connected || $0.connection.status == .connecting
                || $0.connection.status == .reasserting
        }
        guard let manager else { return }
        manager.connection.stopVPNTunnel()
    }
}
