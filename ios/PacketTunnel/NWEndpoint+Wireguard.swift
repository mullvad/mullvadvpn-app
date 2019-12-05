//
//  NWEndpoint+Wireguard.swift
//  PacketTunnel
//
//  Created by pronebird on 24/06/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation
import Network

extension NWEndpoint {
    // String representation of NWEndpoint that supports IPv6-style formatting, i.e [::1]:80
    // which is used by Wireguard
    var wireguardStringRepresentation: String {
        switch self {
        case .hostPort(.name(let hostname, _), let port):
            return "\(hostname):\(port)"
        case .hostPort(.ipv4(let ipv4Address), let port):
            return "\(ipv4Address):\(port)"
        case .hostPort(.ipv6(let ipv6Address), let port):
            return "[\(ipv6Address)]:\(port)"
        default:
            return "\(self)"
        }
    }

}
