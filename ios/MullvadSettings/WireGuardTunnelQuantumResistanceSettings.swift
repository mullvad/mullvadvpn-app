//
//  WireGuardTunnelQuantumResistanceSettings.swift
//  MullvadSettings
//
//  Created by Andrew Bulhak on 2024-02-08.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation

public enum WireGuardTunnelQuantumResistanceState: Codable {
    case automatic
    case on
    case off
}

public struct WireGuardTunnelQuantumResistanceSettings: Codable, Equatable {
    public let state: WireGuardTunnelQuantumResistanceState

    public init(state: WireGuardTunnelQuantumResistanceState = .automatic) {
        self.state = state
    }
}
