//
//  IPAddressRange.swift
//  MullvadTypes
//
//  Created by Mullvad VPN.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network

public struct IPAddressRange: Sendable {
    public let address: AnyIPAddress
    public let networkPrefixLength: UInt8

    public init(address: AnyIPAddress, networkPrefixLength: UInt8) {
        self.address = address
        self.networkPrefixLength = networkPrefixLength
    }
}

extension IPAddressRange: Equatable {
    public static func == (lhs: IPAddressRange, rhs: IPAddressRange) -> Bool {
        lhs.address.rawValue == rhs.address.rawValue && lhs.networkPrefixLength == rhs.networkPrefixLength
    }
}

extension IPAddressRange: Hashable {
    public func hash(into hasher: inout Hasher) {
        hasher.combine(address.rawValue)
        hasher.combine(networkPrefixLength)
    }
}

extension IPAddressRange: Codable {
    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(description)
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let value = try container.decode(String.self)

        if let ipAddressRange = IPAddressRange(from: value) {
            self = ipAddressRange
        } else {
            throw DecodingError.dataCorrupted(
                DecodingError.Context(
                    codingPath: container.codingPath,
                    debugDescription: "Invalid IPAddressRange representation"
                )
            )
        }
    }
}

extension IPAddressRange: CustomStringConvertible {
    public var description: String {
        "\(address)/\(networkPrefixLength)"
    }
}

extension IPAddressRange {
    public init?(from string: String) {
        guard let parsed = IPAddressRange.parseAddressString(string) else { return nil }
        address = parsed.0
        networkPrefixLength = parsed.1
    }

    private static func parseAddressString(_ string: String) -> (AnyIPAddress, UInt8)? {
        // Split "192.168.1.0/24" into address ("192.168.1.0") and prefix length ("24")
        let parts = string.split(separator: "/", maxSplits: 1)
        guard let addressPart = parts.first else { return nil }

        // Parse the address part as either IPv4 or IPv6
        guard let address = AnyIPAddress(String(addressPart)) else { return nil }

        let maxNetworkPrefixLength: UInt8 =
            switch address {
            case .ipv4: 32
            case .ipv6: 128
            }

        // If a prefix length is provided, parse it; otherwise default to the maximum for the address family
        if parts.count > 1 {
            guard let prefixLength = UInt8(parts[1]) else { return nil }
            return (address, min(prefixLength, maxNetworkPrefixLength))
        } else {
            return (address, maxNetworkPrefixLength)
        }
    }

    public func subnetMask() -> IPAddress {
        switch address {
        case .ipv4:
            let mask = networkPrefixLength > 0 ? ~UInt32(0) << (32 - networkPrefixLength) : UInt32(0)
            let bytes = Data([
                UInt8(truncatingIfNeeded: mask >> 24),
                UInt8(truncatingIfNeeded: mask >> 16),
                UInt8(truncatingIfNeeded: mask >> 8),
                UInt8(truncatingIfNeeded: mask >> 0),
            ])
            return IPv4Address(bytes)!

        case .ipv6:
            var bytes = Data(repeating: 0, count: 16)
            for i in 0..<Int(networkPrefixLength / 8) {
                bytes[i] = 0xff
            }
            let nibble = networkPrefixLength % 32
            if nibble != 0 {
                let mask = ~UInt32(0) << (32 - nibble)
                let i = Int(networkPrefixLength / 32 * 4)
                bytes[i + 0] = UInt8(truncatingIfNeeded: mask >> 24)
                bytes[i + 1] = UInt8(truncatingIfNeeded: mask >> 16)
                bytes[i + 2] = UInt8(truncatingIfNeeded: mask >> 8)
                bytes[i + 3] = UInt8(truncatingIfNeeded: mask >> 0)
            }
            return IPv6Address(bytes)!
        }
    }

    public func maskedAddress() -> IPAddress {
        let subnet = subnetMask().rawValue
        var masked = Data(address.rawValue)
        for i in 0..<subnet.count {
            masked[i] &= subnet[i]
        }

        switch address {
        case .ipv4:
            return IPv4Address(masked)!
        case .ipv6:
            return IPv6Address(masked)!
        }
    }
}
