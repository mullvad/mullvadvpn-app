//
//  IPAddress+Codable.swift
//  MullvadTypes
//
//  Created by pronebird on 12/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Network

extension IPv4Address: Codable {
    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let ipString = try container.decode(String.self)

        if let decoded = IPv4Address(ipString) {
            self = decoded
        } else {
            throw DecodingError.dataCorruptedError(
                in: container,
                debugDescription: "Invalid IPv4 representation"
            )
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()

        try container.encode(String(reflecting: self))
    }
}

extension IPv6Address: Codable {
    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let ipString = try container.decode(String.self)

        if let decoded = IPv6Address(ipString) {
            self = decoded
        } else {
            throw DecodingError.dataCorruptedError(
                in: container,
                debugDescription: "Invalid IPv6 representation"
            )
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()

        try container.encode(String(reflecting: self))
    }
}
