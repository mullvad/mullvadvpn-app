//
//  TunnelInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 05/07/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import PacketTunnelCore

protocol TunnelInteractor {
    // MARK: - Tunnel manipulation

    var tunnel: Tunnel? { get }

    func getPersistentTunnels() -> [Tunnel]
    func createNewTunnel() -> Tunnel
    func setTunnel(_ tunnel: Tunnel?, shouldRefreshTunnelState: Bool)

    // MARK: - Tunnel status

    var tunnelStatus: TunnelStatus { get }
    @discardableResult func updateTunnelStatus(_ block: (inout TunnelStatus) -> Void) -> TunnelStatus

    // MARK: - Configuration

    var isConfigurationLoaded: Bool { get }
    var settings: LatestTunnelSettings { get }
    var deviceState: DeviceState { get }

    func setConfigurationLoaded()
    func setSettings(_ settings: LatestTunnelSettings, persist: Bool)
    func setDeviceState(_ deviceState: DeviceState, persist: Bool)
    func handleRestError(_ error: Error)

    func startTunnel()
    func prepareForVPNConfigurationDeletion()
    func selectRelay() throws -> SelectedRelay
}
