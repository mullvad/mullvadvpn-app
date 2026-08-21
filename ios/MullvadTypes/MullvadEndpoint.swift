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
import Network

/// Contains server data needed to connect to a single mullvad endpoint.
public struct MullvadEndpoint: Equatable, Codable, Sendable {
    public let ipv4Relay: IPv4Endpoint
    public let ipv6Relay: IPv6Endpoint?
    public let ipv4Gateway: IPv4Address
    public let ipv6Gateway: IPv6Address
    public let publicKey: Data

    public init(
        ipv4Relay: IPv4Endpoint,
        ipv6Relay: IPv6Endpoint? = nil,
        ipv4Gateway: IPv4Address,
        ipv6Gateway: IPv6Address,
        publicKey: Data
    ) {
        self.ipv4Relay = ipv4Relay
        self.ipv6Relay = ipv6Relay
        self.ipv4Gateway = ipv4Gateway
        self.ipv6Gateway = ipv6Gateway
        self.publicKey = publicKey
    }

    public func override(ipv4Relay: IPv4Endpoint, ipv6Relay: IPv6Endpoint?) -> Self {
        MullvadEndpoint(
            ipv4Relay: ipv4Relay,
            ipv6Relay: ipv6Relay,
            ipv4Gateway: ipv4Gateway,
            ipv6Gateway: ipv6Gateway,
            publicKey: publicKey
        )
    }
}
