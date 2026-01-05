//
//  IPVersionSettings.swift
//  MullvadSettings
//
//  Created by Emīls Piņķis on 2025-12-30.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// IP version preference for relay connections
public enum IPVersion: Codable, Sendable {
    case automatic
    case ipv4
    case ipv6
}

public extension IPVersion {
    /// Returns true if IPv6 should be explicitly used
    var isIPv6: Bool {
        self == .ipv6
    }

    /// Returns true if IPv4 should be explicitly used
    var isIPv4: Bool {
        self == .ipv4
    }

    /// Returns true if the version should be automatically selected
    var isAutomatic: Bool {
        self == .automatic
    }
}
