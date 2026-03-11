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
    case always
    case never
    case whenNeeded

    // is multihop explicitly selected by the user?
    // as this is now a tristate value, this will presumably largely go away
    public var isUserSelected: Bool {
        get {
            self == .always
        }
        set {
            self = newValue ? .always : .never
        }
    }
}
