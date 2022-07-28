//
//  PacketTunnelStatus.swift
//  PacketTunnelStatus
//
//  Created by pronebird on 27/07/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Struct describing packet tunnel process status.
struct PacketTunnelStatus: Codable, Equatable {
    /// Flag indicating whether network is reachable.
    var isNetworkReachable: Bool = true

    /// When the packet tunnel started connecting.
    var connectingDate: Date?

    /// Current relay.
    var tunnelRelay: PacketTunnelRelay?
}

/// Struct holding tunnel relay information.
struct PacketTunnelRelay: Codable, Equatable {
    /// IPv4 relay endpoint.
    let ipv4Relay: IPv4Endpoint

    /// IPv6 relay endpoint.
    let ipv6Relay: IPv6Endpoint?

    /// Relay hostname.
    let hostname: String

    /// Relay location.
    let location: Location
}
