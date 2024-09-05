//
//  ServerRelaysResponse.swift
//  MullvadREST
//
//  Created by pronebird on 27/07/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import Network

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

    public struct BridgeRelay: Codable, Equatable {
        public let hostname: String
        public let active: Bool
        public let owned: Bool
        public let location: String
        public let provider: String
        public let ipv4AddrIn: IPv4Address
        public let weight: UInt64
        public let includeInCountry: Bool
        public var daita: Bool?

        public func override(ipv4AddrIn: IPv4Address?) -> Self {
            return BridgeRelay(
                hostname: hostname,
                active: active,
                owned: owned,
                location: location,
                provider: provider,
                ipv4AddrIn: ipv4AddrIn ?? self.ipv4AddrIn,
                weight: weight,
                includeInCountry: includeInCountry
            )
        }
    }

    public struct ServerRelay: Codable, Equatable {
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
        public let daita: Bool?

        public func override(ipv4AddrIn: IPv4Address?, ipv6AddrIn: IPv6Address?) -> Self {
            return ServerRelay(
                hostname: hostname,
                active: active,
                owned: owned,
                location: location,
                provider: provider,
                weight: weight,
                ipv4AddrIn: ipv4AddrIn ?? self.ipv4AddrIn,
                ipv6AddrIn: ipv6AddrIn ?? self.ipv6AddrIn,
                publicKey: publicKey,
                includeInCountry: includeInCountry,
                daita: daita
            )
        }

        public func override(daita: Bool) -> Self {
            return ServerRelay(
                hostname: hostname,
                active: active,
                owned: owned,
                location: location,
                provider: provider,
                weight: weight,
                ipv4AddrIn: ipv4AddrIn,
                ipv6AddrIn: ipv6AddrIn,
                publicKey: publicKey,
                includeInCountry: includeInCountry,
                daita: daita
            )
        }
    }

    public struct ServerWireguardTunnels: Codable, Equatable {
        public let ipv4Gateway: IPv4Address
        public let ipv6Gateway: IPv6Address
        public let portRanges: [[UInt16]]
        public let relays: [ServerRelay]

        public init(ipv4Gateway: IPv4Address, ipv6Gateway: IPv6Address, portRanges: [[UInt16]], relays: [ServerRelay]) {
            self.ipv4Gateway = ipv4Gateway
            self.ipv6Gateway = ipv6Gateway
            self.portRanges = portRanges
            self.relays = relays
        }
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

        public init(locations: [String: ServerLocation], wireguard: ServerWireguardTunnels, bridge: ServerBridges) {
            self.locations = locations
            self.wireguard = wireguard
            self.bridge = bridge
        }
    }
}
