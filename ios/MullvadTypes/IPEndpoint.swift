//
//  IPEndpoint.swift
//  MullvadVPN
//
//  Created by pronebird on 06/12/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import protocol Network.IPAddress
import struct Network.IPv4Address
import struct Network.IPv6Address

public struct IPv4Endpoint: Hashable, Equatable, Codable, CustomStringConvertible {
    public let ip: IPv4Address
    public let port: UInt16

    public init(ip: IPv4Address, port: UInt16) {
        self.ip = ip
        self.port = port
    }

    public init?<S>(string: S) where S: StringProtocol {
        let components = string.split(
            separator: ":",
            maxSplits: 2,
            omittingEmptySubsequences: false
        )

        if components.count == 2, let parsedIP = IPv4Address(String(components[0])),
           let parsedPort = UInt16(components[1])
        {
            ip = parsedIP
            port = parsedPort
        } else {
            return nil
        }
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let string = try container.decode(String.self)

        if let parsedAddress = IPv4Endpoint(string: string) {
            self = parsedAddress
        } else {
            throw DecodingError.dataCorruptedError(
                in: container,
                debugDescription: "Cannot parse the IPv4 endpoint"
            )
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()

        try container.encode("\(self)")
    }

    public var description: String {
        return "\(ip):\(port)"
    }

    public static func == (lhs: IPv4Endpoint, rhs: IPv4Endpoint) -> Bool {
        return lhs.ip.rawValue == rhs.ip.rawValue && lhs.port == rhs.port
    }
}

public struct IPv6Endpoint: Hashable, Equatable, Codable, CustomStringConvertible {
    public let ip: IPv6Address
    public let port: UInt16

    public init(ip: IPv6Address, port: UInt16) {
        self.ip = ip
        self.port = port
    }

    public init?<S>(string: S) where S: StringProtocol {
        guard let lastColon = string.lastIndex(of: ":"), lastColon != string.endIndex else {
            return nil
        }

        let portIndex = string.index(after: lastColon)
        let addressString = string[..<lastColon]
        let portString = string[portIndex...]

        guard addressString.first == "[", addressString.last == "]" else {
            return nil
        }

        let ipv6AddressString = String(addressString.dropFirst().dropLast())

        if let parsedIP = IPv6Address(ipv6AddressString), let parsedPort = UInt16(portString) {
            ip = parsedIP
            port = parsedPort
        } else {
            return nil
        }
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let string = try container.decode(String.self)

        if let parsedAddress = IPv6Endpoint(string: string) {
            self = parsedAddress
        } else {
            throw DecodingError.dataCorruptedError(
                in: container,
                debugDescription: "Cannot parse the IPv6 endpoint"
            )
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()

        try container.encode("\(self)")
    }

    public var description: String {
        return "[\(ip)]:\(port)"
    }

    public static func == (lhs: IPv6Endpoint, rhs: IPv6Endpoint) -> Bool {
        return lhs.ip.rawValue == rhs.ip.rawValue && lhs.port == rhs.port
    }
}

public enum AnyIPEndpoint: Hashable, Equatable, Codable, CustomStringConvertible {
    case ipv4(IPv4Endpoint)
    case ipv6(IPv6Endpoint)

    public var ip: IPAddress {
        switch self {
        case let .ipv4(ipv4Endpoint):
            return ipv4Endpoint.ip
        case let .ipv6(ipv6Endpoint):
            return ipv6Endpoint.ip
        }
    }

    public var port: UInt16 {
        switch self {
        case let .ipv4(ipv4Endpoint):
            return ipv4Endpoint.port
        case let .ipv6(ipv6Endpoint):
            return ipv6Endpoint.port
        }
    }

    public init?<S>(string: S) where S: StringProtocol {
        if let ipv4Endpoint = IPv4Endpoint(string: string) {
            self = .ipv4(ipv4Endpoint)
        } else if let ipv6Endpoint = IPv6Endpoint(string: string) {
            self = .ipv6(ipv6Endpoint)
        } else {
            return nil
        }
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let string = try container.decode(String.self)

        if let ipv4Endpoint = IPv4Endpoint(string: string) {
            self = .ipv4(ipv4Endpoint)
        } else if let ipv6Endpoint = IPv6Endpoint(string: string) {
            self = .ipv6(ipv6Endpoint)
        } else {
            throw DecodingError.dataCorruptedError(
                in: container,
                debugDescription: "Cannot parse the endpoint"
            )
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()

        try container.encode("\(self)")
    }

    public var description: String {
        switch self {
        case let .ipv4(ipv4Endpoint):
            return "\(ipv4Endpoint)"
        case let .ipv6(ipv6Endpoint):
            return "\(ipv6Endpoint)"
        }
    }

    public static func == (lhs: AnyIPEndpoint, rhs: AnyIPEndpoint) -> Bool {
        switch (lhs, rhs) {
        case let (.ipv4(lhsEndpoint), .ipv4(rhsEndpoint)):
            return lhsEndpoint == rhsEndpoint

        case let (.ipv6(lhsEndpoint), .ipv6(rhsEndpoint)):
            return lhsEndpoint == rhsEndpoint

        default:
            return false
        }
    }
}
