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

    func setDeviceState(_ deviceState: DeviceState, persist: Bool) {
        self.deviceState = deviceState
    }

    func removeLastUsedAccount() {}

    func handleRestError(_ error: Error) {}

    func startTunnel() {}

    func prepareForVPNConfigurationDeletion() {}

    struct NotImplementedError: Error {}

    func selectRelays() throws -> SelectedRelays {
        throw NotImplementedError()
    }
}
