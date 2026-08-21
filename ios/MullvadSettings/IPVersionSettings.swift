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

/// IP version preference for relay connections
public enum IPVersion: Codable, Sendable, CaseIterable {
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
