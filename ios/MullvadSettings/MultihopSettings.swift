//
//  MultihopSettings.swift
//  MullvadSettings
//
//  Created by Mojgan on 2024-04-26.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

/// Whether Multi-hop is enabled
public enum MultihopState: Codable, Sendable {
    case on
    case off

    public var isEnabled: Bool {
        get {
            self == .on
        }
        set {
            self = newValue ? .on : .off
        }
    }
}
