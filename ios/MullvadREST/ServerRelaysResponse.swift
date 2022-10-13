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
    public struct ServerLocation: Codable {
        public let country: String
        public let city: String
        public let latitude: Double
        public let longitude: Double
    }

    public struct ServerRelay: Codable {
        public let hostname: String
        public let active: Bool
        public let owned: Bool
        public let location: String
        public let provider: String
        public let weight: UInt64
        public let ipv4AddrIn: IPv4Address
        public let ipv6AddrIn: IPv6Address
        public let publicKey: Data
        public let includeInCountry: Bool
    }

    public struct ServerWireguardTunnels: Codable {
        public let ipv4Gateway: IPv4Address
        public let ipv6Gateway: IPv6Address
        public let portRanges: [[UInt16]]
        public let relays: [ServerRelay]
    }

    public struct ServerRelaysResponse: Codable {
        public let locations: [String: ServerLocation]
        public let wireguard: ServerWireguardTunnels
    }
}
