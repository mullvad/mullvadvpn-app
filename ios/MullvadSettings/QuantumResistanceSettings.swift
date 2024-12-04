//
//  QuantumResistanceSettings.swift
//  MullvadSettings
//
//  Created by Andrew Bulhak on 2024-02-08.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation

public enum TunnelQuantumResistance: Codable, Sendable {
    case automatic
    case on
    case off
}

public extension TunnelQuantumResistance {
    /// A single source of truth for whether the current state counts as on
    var isEnabled: Bool {
        self == .on
    }
}
