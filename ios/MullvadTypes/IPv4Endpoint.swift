//
//  IPEndpoint.swift
//  MullvadTypes
//
//  Created by pronebird on 06/12/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import struct Network.IPv4Address

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
