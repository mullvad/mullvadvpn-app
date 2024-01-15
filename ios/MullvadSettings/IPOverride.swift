//
//  IPOverride.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-01-16.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Network

public struct RelayOverrides: Codable {
    public let overrides: [IPOverride]

    private enum CodingKeys: String, CodingKey {
        case overrides = "relay_overrides"
    }
}

public struct IPOverrideFormatError: LocalizedError {
    public let errorDescription: String?
}

public struct IPOverride: Codable, Equatable {
    public let hostname: String
    public var ipv4Address: IPv4Address?
    public var ipv6Address: IPv6Address?

    private enum CodingKeys: String, CodingKey {
        case hostname
        case ipv4Address = "ipv4_addr_in"
        case ipv6Address = "ipv6_addr_in"
    }

    public init(hostname: String, ipv4Address: IPv4Address?, ipv6Address: IPv6Address?) throws {
        self.hostname = hostname
        self.ipv4Address = ipv4Address
        self.ipv6Address = ipv6Address

        if self.ipv4Address.isNil && self.ipv6Address.isNil {
            throw IPOverrideFormatError(errorDescription: "ipv4Address and ipv6Address cannot both be nil.")
        }
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)

        self.hostname = try container.decode(String.self, forKey: .hostname)
        self.ipv4Address = try container.decodeIfPresent(IPv4Address.self, forKey: .ipv4Address)
        self.ipv6Address = try container.decodeIfPresent(IPv6Address.self, forKey: .ipv6Address)

        if self.ipv4Address.isNil && self.ipv6Address.isNil {
            throw IPOverrideFormatError(errorDescription: "ipv4Address and ipv6Address cannot both be nil.")
        }
    }
}
