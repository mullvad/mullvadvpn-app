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
    case on
    case off
    case whenNeeded

    // is multihop explicitly selected by the user?
    public var isUserSelected: Bool {
        get {
            self == .on
        }
        set {
            // once .whenNeeded is used, the .off value below should
            // perhaps be replaced with .whenNeeded
            self = newValue ? .on : .off
        }
    }
}
