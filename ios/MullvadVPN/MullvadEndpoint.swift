//
//  MullvadEndpoint.swift
//  MullvadVPN
//
//  Created by pronebird on 12/06/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation
import Network

/// Contains server data needed to connect to a single mullvad endpoint
struct MullvadEndpoint {
    let ipv4Relay: NWEndpoint
    let ipv6Relay: NWEndpoint?
    let ipv4Gateway: IPv4Address
    let ipv6Gateway: IPv6Address
    let publicKey: Data
}
