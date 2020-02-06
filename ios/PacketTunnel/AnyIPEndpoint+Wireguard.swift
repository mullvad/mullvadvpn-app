//
//  AnyIPEndpoint+Wireguard.swift
//  PacketTunnel
//
//  Created by pronebird on 24/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network

extension AnyIPEndpoint {

    /// String representation that supports IPv6-style formatting (i.e [::1]:80) used by Wireguard
    var wireguardStringRepresentation: String {
        switch self {
        case .ipv4(let ipv4Endpoint):
            return "\(ipv4Endpoint.ip):\(ipv4Endpoint.port)"
        case .ipv6(let ipv6Endpoint):
            return "[\(ipv6Endpoint.ip)]:\(ipv6Endpoint.port)"
        }
    }

}
