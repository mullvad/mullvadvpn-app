//
//  IPEndpoint.swift
//  MullvadVPN
//
//  Created by pronebird on 06/12/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import struct Network.IPv4Address
import struct Network.IPv6Address
import protocol Network.IPAddress

struct IPv4Endpoint: Hashable, Equatable, Codable, CustomStringConvertible {
    let ip: IPv4Address
    let port: UInt16

    init(ip: IPv4Address, port: UInt16) {
        self.ip = ip
        self.port = port
    }

    init?<S>(string: S) where S: StringProtocol {
        let components = string.split(separator: ":", maxSplits: 2, omittingEmptySubsequences: false)

        if components.count == 2, let parsedIP = IPv4Address(String(components[0])), let parsedPort = UInt16(components[1]) {
            ip = parsedIP
            port = parsedPort
        } else {
            return nil
        }
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let string = try container.decode(String.self)

        if let parsedAddress = IPv4Endpoint(string: string) {
            self = parsedAddress
        } else {
            throw DecodingError.dataCorruptedError(in: container, debugDescription: "Cannot parse the IPv4 endpoint")
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()

        try container.encode("\(self)")
    }

    var description: String {
        return "\(ip):\(port)"
    }

    static func == (lhs: IPv4Endpoint, rhs: IPv4Endpoint) -> Bool {
        return lhs.ip.rawValue == rhs.ip.rawValue && lhs.port == rhs.port
    }
}

struct IPv6Endpoint: Hashable, Equatable, Codable, CustomStringConvertible {
    let ip: IPv6Address
    let port: UInt16

    init(ip: IPv6Address, port: UInt16) {
        self.ip = ip
        self.port = port
    }

    init?<S>(string: S) where S: StringProtocol {
        guard let lastColon = string.lastIndex(of: ":"), lastColon != string.endIndex else {
            return nil
        }

        let portIndex = string.index(after: lastColon)
        let addressString = string[..<lastColon]
        let portString = string[portIndex...]

        guard addressString.first == "[" && addressString.last == "]" else {
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

    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let string = try container.decode(String.self)

        if let parsedAddress = IPv6Endpoint(string: string) {
            self = parsedAddress
        } else {
            throw DecodingError.dataCorruptedError(in: container, debugDescription: "Cannot parse the IPv6 endpoint")
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()

        try container.encode("\(self)")
    }

    var description: String {
        return "[\(ip)]:\(port)"
    }

    static func == (lhs: IPv6Endpoint, rhs: IPv6Endpoint) -> Bool {
        return lhs.ip.rawValue == rhs.ip.rawValue && lhs.port == rhs.port
    }
}

enum AnyIPEndpoint: Hashable, Equatable, Codable, CustomStringConvertible {
    case ipv4(IPv4Endpoint)
    case ipv6(IPv6Endpoint)

    var ip: IPAddress {
        switch self {
        case .ipv4(let ipv4Endpoint):
            return ipv4Endpoint.ip
        case .ipv6(let ipv6Endpoint):
            return ipv6Endpoint.ip
        }
    }

    var port: UInt16 {
        switch self {
        case .ipv4(let ipv4Endpoint):
            return ipv4Endpoint.port
        case .ipv6(let ipv6Endpoint):
            return ipv6Endpoint.port
        }
    }

    init?<S>(string: S) where S: StringProtocol {
        if let ipv4Endpoint = IPv4Endpoint(string: string) {
            self = .ipv4(ipv4Endpoint)
        } else if let ipv6Endpoint = IPv6Endpoint(string: string) {
            self = .ipv6(ipv6Endpoint)
        } else {
            return nil
        }
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let string = try container.decode(String.self)

        if let ipv4Endpoint = IPv4Endpoint(string: string) {
            self = .ipv4(ipv4Endpoint)
        } else if let ipv6Endpoint = IPv6Endpoint(string: string) {
            self = .ipv6(ipv6Endpoint)
        } else {
            throw DecodingError.dataCorruptedError(in: container, debugDescription: "Cannot parse the endpoint")
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()

        try container.encode("\(self)")
    }

    var description: String {
        switch self {
        case .ipv4(let ipv4Endpoint):
            return "\(ipv4Endpoint)"
        case .ipv6(let ipv6Endpoint):
            return "\(ipv6Endpoint)"
        }
    }

    static func == (lhs: AnyIPEndpoint, rhs: AnyIPEndpoint) -> Bool {
        switch (lhs, rhs) {
        case (.ipv4(let lhsEndpoint), .ipv4(let rhsEndpoint)):
            return lhsEndpoint == rhsEndpoint

        case (.ipv6(let lhsEndpoint), .ipv6(let rhsEndpoint)):
            return lhsEndpoint == rhsEndpoint

        default:
            return false
        }
    }
}
