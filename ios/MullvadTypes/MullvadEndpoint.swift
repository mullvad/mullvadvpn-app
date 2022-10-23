//
//  MullvadEndpoint.swift
//  MullvadTypes
//
//  Created by pronebird on 12/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import struct Network.IPv4Address
import struct Network.IPv6Address

/// Contains server data needed to connect to a single mullvad endpoint.
public struct MullvadEndpoint: Equatable, Codable {
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
}
