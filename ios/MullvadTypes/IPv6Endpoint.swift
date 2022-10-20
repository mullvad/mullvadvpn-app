//
//  IPv6Endpoint.swift
//  MullvadTypes
//
//  Created by pronebird on 20/10/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import struct Network.IPv6Address

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
