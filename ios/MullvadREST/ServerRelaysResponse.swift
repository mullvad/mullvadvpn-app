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

extension REST {
    public struct ServerLocation: Codable {
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

    public struct BridgeRelay: Codable {
        public let hostname: String
        public let active: Bool
        public let owned: Bool
        public let location: String
        public let provider: String
        public let ipv4AddrIn: IPv4Address
        public let weight: UInt64
        public let includeInCountry: Bool
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

        public init(
            hostname: String,
            active: Bool,
            owned: Bool,
            location: String,
            provider: String,
            weight: UInt64,
            ipv4AddrIn: IPv4Address,
            ipv6AddrIn: IPv6Address,
            publicKey: Data,
            includeInCountry: Bool
        ) {
            self.hostname = hostname
            self.active = active
            self.owned = owned
            self.location = location
            self.provider = provider
            self.weight = weight
            self.ipv4AddrIn = ipv4AddrIn
            self.ipv6AddrIn = ipv6AddrIn
            self.publicKey = publicKey
            self.includeInCountry = includeInCountry
        }
    }

    public struct ServerWireguardTunnels: Codable {
        public let ipv4Gateway: IPv4Address
        public let ipv6Gateway: IPv6Address
        public let portRanges: [[UInt16]]
        public let relays: [ServerRelay]

        public init(
            ipv4Gateway: IPv4Address,
            ipv6Gateway: IPv6Address,
            portRanges: [[UInt16]],
            relays: [REST.ServerRelay]
        ) {
            self.ipv4Gateway = ipv4Gateway
            self.ipv6Gateway = ipv6Gateway
            self.portRanges = portRanges
            self.relays = relays
        }
    }

    public struct ServerShadowsocks: Codable {
        public let `protocol`: String
        public let port: UInt16
        public let cipher: String
        public let password: String

        public init(protocol: String, port: UInt16, cipher: String, password: String) {
            self.protocol = `protocol`
            self.port = port
            self.cipher = cipher
            self.password = password
        }
    }

    public struct ServerBridges: Codable {
        public let shadowsocks: [ServerShadowsocks]
        public let relays: [BridgeRelay]

        public init(shadowsocks: [REST.ServerShadowsocks], relays: [BridgeRelay] ) {
            self.shadowsocks = shadowsocks
            self.relays = relays
        }
    }

    public struct ServerRelaysResponse: Codable {
        public let locations: [String: ServerLocation]
        public let wireguard: ServerWireguardTunnels
        public let bridge: ServerBridges

        public init(
            locations: [String: REST.ServerLocation],
            wireguard: REST.ServerWireguardTunnels,
            bridge: ServerBridges
        ) {
            self.locations = locations
            self.wireguard = wireguard
            self.bridge = bridge
        }
    }
}
