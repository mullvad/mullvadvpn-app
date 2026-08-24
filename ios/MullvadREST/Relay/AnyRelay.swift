// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadTypes
import Network

public protocol AnyRelay {
    var hostname: String { get }
    var owned: Bool { get }
    var location: REST.LocationIdentifier { get }
    var provider: String { get }
    var weight: UInt64 { get }
    var active: Bool { get }
    var includeInCountry: Bool { get }
    var daita: Bool? { get }
    var isIPOverridden: Bool? { get }
    func override(ipv4AddrIn: IPv4Address?, ipv6AddrIn: IPv6Address?) -> Self
}

extension REST.BridgeRelay {
    public func override(ipv4AddrIn: IPv4Address?, ipv6AddrIn: IPv6Address?) -> REST.BridgeRelay {
        override(ipv4AddrIn: ipv4AddrIn)
    }
}
