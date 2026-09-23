// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
    var onSetTunnel: (((any TunnelProtocol)?, Bool) -> Void)?
    var onSetSettings: ((LatestTunnelSettings, Bool) -> Void)?
    var onSetDeviceState: ((DeviceState, Bool) -> Void)?
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

    func getTunnel() async -> (any TunnelProtocol)? {
        tunnel
    }

    func getTunnelStatus() async -> TunnelStatus {
        tunnelStatus
    }

    func getDeviceState() async -> DeviceState {
        deviceState
    }

    func getPersistentTunnel() async -> (any TunnelProtocol)? {
        tunnel
    }

    func createNewTunnel() async -> any TunnelProtocol {
        MockTunnel(
            tunnelProvider: SimulatorTunnelProviderManager(),
            backgroundTaskProvider: backgroundTaskProvider
        )
    }

    func setTunnel(_ tunnel: (any TunnelProtocol)?, shouldRefreshTunnelState: Bool) async {
        onSetTunnel?(tunnel, shouldRefreshTunnelState)
        self.tunnel = tunnel
    }

    var tunnelStatus: TunnelStatus

    @discardableResult func updateTunnelStatus(_ block: @Sendable (inout TunnelStatus) -> Void) async -> TunnelStatus {
        var tunnelStatus = self.tunnelStatus
        block(&tunnelStatus)
        onUpdateTunnelStatus?(tunnelStatus)
        self.tunnelStatus = tunnelStatus
        return tunnelStatus
    }

    func setConfigurationLoaded() async {
        isConfigurationLoaded = true
    }

    func setSettings(_ settings: LatestTunnelSettings, persist: Bool) async {
        onSetSettings?(settings, persist)
        self.settings = settings
    }

    func setDeviceState(_ deviceState: DeviceState, persist: Bool) async {
        onSetDeviceState?(deviceState, persist)
        self.deviceState = deviceState
    }

    func setLastUsedAccount(_ accountNumber: String) async {}

    func removeLastUsedAccount() async {}

    func handleRestError(_ error: Error) async {}

    func startTunnel() async {}

    func prepareForVPNConfigurationDeletion() async {}

    struct NotImplementedError: Error {}

    func selectRelays() async throws -> SelectedRelays {
        throw NotImplementedError()
    }
}
