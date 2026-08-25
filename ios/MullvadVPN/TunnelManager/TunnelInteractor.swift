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
import PacketTunnelCore

protocol TunnelInteractor {

    // MARK: - Tunnel manipulation
    
    var tunnel: (any TunnelProtocol)? { get }
    var backgroundTaskProvider: BackgroundTaskProviding { get }
    func getPersistentTunnels() -> [any TunnelProtocol]
    func createNewTunnel() -> any TunnelProtocol
    func setTunnel(
        _ tunnel: (any TunnelProtocol)?,
        shouldRefreshTunnelState: Bool
    ) async

    // MARK: - Tunnel status

    var tunnelStatus: TunnelStatus { get }

    @discardableResult
    func updateTunnelStatus(
        _ block: @Sendable (inout TunnelStatus) -> Void
    ) async -> TunnelStatus

    // MARK: - Configuration

    var isConfigurationLoaded: Bool { get }

    var settings: LatestTunnelSettings { get }

    var deviceState: DeviceState { get }

    func setConfigurationLoaded() async

    func setSettings(
        _ settings: LatestTunnelSettings,
        persist: Bool
    ) async

    func setDeviceState(
        _ deviceState: DeviceState,
        persist: Bool
    ) async

    func removeLastUsedAccount() async

    func handleRestError(_ error: Error) async

    func startTunnel() async

    func prepareForVPNConfigurationDeletion() async

    func selectRelays() async throws -> SelectedRelays
}
