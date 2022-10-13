//
//  MullvadEndpoint.swift
//  MullvadVPN
//
//  Created by pronebird on 12/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import Network

/// Contains server data needed to connect to a single mullvad endpoint
struct MullvadEndpoint: Equatable, Codable {
    let ipv4Relay: IPv4Endpoint
    let ipv6Relay: IPv6Endpoint?
    let ipv4Gateway: IPv4Address
    let ipv6Gateway: IPv6Address
    let publicKey: Data
}
