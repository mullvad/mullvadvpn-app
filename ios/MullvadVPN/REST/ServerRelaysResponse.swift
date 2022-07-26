//
//  ServerRelaysResponse.swift
//  ServerRelaysResponse
//
//  Created by pronebird on 27/07/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import struct Network.IPv4Address
import struct Network.IPv6Address

extension REST {
    struct ServerLocation: Codable {
        let country: String
        let city: String
        let latitude: Double
        let longitude: Double
    }

    struct ServerRelay: Codable, Equatable {
        let hostname: String
        let active: Bool
        let owned: Bool
        let location: String
        let provider: String
        let weight: UInt64
        let ipv4AddrIn: IPv4Address
        let ipv6AddrIn: IPv6Address
        let publicKey: Data
        let includeInCountry: Bool
    }

    struct ServerWireguardTunnels: Codable {
        let ipv4Gateway: IPv4Address
        let ipv6Gateway: IPv6Address
        let portRanges: [[UInt16]]
        let relays: [ServerRelay]
    }

    struct ServerRelaysResponse: Codable {
        let locations: [String: ServerLocation]
        let wireguard: ServerWireguardTunnels
    }
}
