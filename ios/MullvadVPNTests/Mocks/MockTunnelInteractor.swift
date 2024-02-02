//
//  MockTunnelInteractor.swift
//  MullvadVPNTests
//
//  Created by Andrew Bulhak on 2024-02-02.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import PacketTunnelCore

// this is still very minimal, and will be fleshed out as needed. 
struct MockTunnelInteractor: TunnelInteractor {
    var onUpdateTunnelStatus: ((TunnelStatus)->Void)?
    
    
    var tunnel: (TunnelProtocol)?
    
    func getPersistentTunnels() -> [TunnelProtocol] {
        return []
    }
    
    func createNewTunnel() -> TunnelProtocol {
        fatalError()
    }
    
    func setTunnel(_ tunnel: (TunnelProtocol)?, shouldRefreshTunnelState: Bool) {
        
    }
    
    var tunnelStatus: TunnelStatus =
        TunnelStatus()
    
    func updateTunnelStatus(_ block: (inout TunnelStatus) -> Void) -> TunnelStatus {
        var tunnelStatus = self.tunnelStatus
        block(&tunnelStatus)
        onUpdateTunnelStatus?(tunnelStatus)
        return tunnelStatus
    }
    
    var isConfigurationLoaded: Bool
    
    var settings: MullvadSettings.LatestTunnelSettings
    
    var deviceState: MullvadSettings.DeviceState
    
    func setConfigurationLoaded() {
    }
    
    func setSettings(_ settings: MullvadSettings.LatestTunnelSettings, persist: Bool) {
    }
    
    func setDeviceState(_ deviceState: MullvadSettings.DeviceState, persist: Bool) {
    }
    
    func removeLastUsedAccount() {
    }
    
    func handleRestError(_ error: Error) {
    }
    
    func startTunnel() {
    }
    
    func prepareForVPNConfigurationDeletion() {
    }
    
    func selectRelay() throws -> PacketTunnelCore.SelectedRelay {
        fatalError()
    }
}
