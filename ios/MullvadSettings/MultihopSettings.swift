//
//  MultihopSettings.swift
//  MullvadSettings
//
//  Created by Mojgan on 2024-04-26.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

/// Whether Multi-hop is enabled
public enum MultihopState: Codable, Sendable {
    // the legacy settings: these will be migrated away from and removed
    case on
    case off
    // the new settings
    case always
    case never
    case whenNeeded

    // is multihop explicitly selected by the user?
    public var isUserSelected: Bool {
        get {
            self == .on || self == .always
        }
        set {
            // once .whenNeeded is used, the .off value below should
            // perhaps be replaced with .whenNeeded
            self = newValue ? .on : .off
        }
    }
}
