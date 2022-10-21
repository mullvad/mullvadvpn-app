//
//  PacketTunnelRelay.swift
//  MullvadTypes
//
//  Created by pronebird on 21/10/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Struct holding tunnel relay information.
public struct PacketTunnelRelay: Codable, Equatable {
    /// IPv4 relay endpoint.
    public let ipv4Relay: IPv4Endpoint

    /// IPv6 relay endpoint.
    public let ipv6Relay: IPv6Endpoint?

    /// Relay hostname.
    public let hostname: String

    /// Relay location.
    public let location: Location

    public init(
        ipv4Relay: IPv4Endpoint,
        ipv6Relay: IPv6Endpoint? = nil,
        hostname: String,
        location: Location
    ) {
        self.ipv4Relay = ipv4Relay
        self.ipv6Relay = ipv6Relay
        self.hostname = hostname
        self.location = location
    }
}
