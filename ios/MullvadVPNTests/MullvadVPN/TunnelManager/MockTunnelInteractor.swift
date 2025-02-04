//
//  MockTunnelInteractor.swift
//  MullvadVPNTests
//
//  Created by Andrew Bulhak on 2024-02-02.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadSettings
import MullvadTypes

// this is still very minimal, and will be fleshed out as needed.
final class MockTunnelInteractor: TunnelInteractor, @unchecked Sendable {
    var isConfigurationLoaded: Bool

    var settings: LatestTunnelSettings

    var deviceState: DeviceState

    var onUpdateTunnelStatus: ((TunnelStatus) -> Void)?

    var tunnel: (any TunnelProtocol)?

    var backgroundTaskProvider: BackgroundTaskProviding {
        UIApplicationStub()
    }

    init(
        isConfigurationLoaded: Bool,
        settings: LatestTunnelSettings,
        deviceState: DeviceState,
        onUpdateTunnelStatus: ((TunnelStatus) -> Void)? = nil
    ) {
        self.isConfigurationLoaded = isConfigurationLoaded
        self.settings = settings
        self.deviceState = deviceState
        self.onUpdateTunnelStatus = onUpdateTunnelStatus
        self.tunnel = nil
        self.tunnelStatus = TunnelStatus()
    }

    func getPersistentTunnels() -> [any TunnelProtocol] {
        return []
    }

    func createNewTunnel() -> any TunnelProtocol {
        return MockTunnel(
            tunnelProvider: SimulatorTunnelProviderManager(),
            backgroundTaskProvider: backgroundTaskProvider
        )
    }

    func setTunnel(_ tunnel: (any TunnelProtocol)?, shouldRefreshTunnelState: Bool) {
        self.tunnel = tunnel
    }

    var tunnelStatus: TunnelStatus

    func updateTunnelStatus(_ block: (inout TunnelStatus) -> Void) -> TunnelStatus {
        var tunnelStatus = self.tunnelStatus
        block(&tunnelStatus)
        onUpdateTunnelStatus?(tunnelStatus)
        return tunnelStatus
    }

    func setConfigurationLoaded() {}

    func setSettings(_ settings: LatestTunnelSettings, persist: Bool) {}

    func setDeviceState(_ deviceState: DeviceState, persist: Bool) {}

    func removeLastUsedAccount() {}

    func handleRestError(_ error: Error) {}

    func startTunnel() {}

    func prepareForVPNConfigurationDeletion() {}

    struct NotImplementedError: Error {}

    func selectRelays() throws -> SelectedRelays {
        throw NotImplementedError()
    }
}
