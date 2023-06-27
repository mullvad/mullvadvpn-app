//
//  ServerRelaysResponse.swift
//  MullvadREST
//
//  Created by pronebird on 27/07/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import struct Network.IPv4Address
import struct Network.IPv6Address

public protocol Relay: Codable, Equatable {
    var hostname: String { get }
    var active: Bool { get }
    var owned: Bool { get }
    var location: String { get }
    var provider: String { get }
    var ipv4AddrIn: IPv4Address { get }
    var ipv6AddrIn: IPv6Address { get }
    var weight: UInt64 { get }
    var publicKey: Data { get }
    var includeInCountry: Bool { get }
}

extension REST {
    public struct ServerLocation: Codable, Equatable {
        public let country: String
        public let city: String
        public let latitude: Double
        public let longitude: Double

        public init(country: String, city: String, latitude: Double, longitude: Double) {
            self.country = country
            self.city = city
            self.latitude = latitude
            self.longitude = longitude
        }
    }

    public struct BridgeRelay: Relay {
        public let hostname: String
        public let active: Bool
        public let owned: Bool
        public let location: String
        public let provider: String
        public let ipv4AddrIn: IPv4Address
        public let weight: UInt64
        public let includeInCountry: Bool
        // Unused, therefore not included in `CodingKeys`
        public let ipv6AddrIn: IPv6Address = .loopback
        // Unused, therefore not included in `CodingKeys`
        public let publicKey = Data()

        private enum CodingKeys: String, CodingKey {
            case hostname, active, owned, location, provider, ipv4AddrIn, weight, includeInCountry
        }
    }

    public struct ServerRelay: Relay {
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

    public struct ServerWireguardTunnels: Codable, Equatable {
        public let ipv4Gateway: IPv4Address
        public let ipv6Gateway: IPv6Address
        public let portRanges: [[UInt16]]
        public let relays: [ServerRelay]
    }

    public struct ServerShadowsocks: Codable, Equatable {
        public let `protocol`: String
        public let port: UInt16
        public let cipher: String
        public let password: String
    }

    public struct ServerBridges: Codable, Equatable {
        public let shadowsocks: [ServerShadowsocks]
        public let relays: [BridgeRelay]
    }

    public struct ServerRelaysResponse: Codable, Equatable {
        public let locations: [String: ServerLocation]
        public let wireguard: ServerWireguardTunnels
        public let bridge: ServerBridges
    }
}
