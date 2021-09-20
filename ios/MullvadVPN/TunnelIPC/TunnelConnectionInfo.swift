//
//  TunnelConnectionInfo.swift
//  TunnelConnectionInfo
//
//  Created by pronebird on 27/07/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// A struct that holds basic information regarding the tunnel connection.
struct TunnelConnectionInfo: Codable, Equatable {
    let ipv4Relay: IPv4Endpoint
    let ipv6Relay: IPv6Endpoint?
    let hostname: String
    let location: Location
}
