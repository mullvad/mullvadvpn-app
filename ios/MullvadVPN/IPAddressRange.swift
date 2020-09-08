//
//  IPAddressRange.swift
//  MullvadVPN
//
//  Created by pronebird on 20/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//  Copyright © 2018-2019 WireGuard LLC. All Rights Reserved.
//

import Foundation
import Network

/// A struct describing an IP address range
struct IPAddressRange {
    let address: IPAddress
    let networkPrefixLength: UInt8

    init(address: IPAddress, networkPrefixLength: UInt8) {
        self.address = address
        self.networkPrefixLength = min(networkPrefixLength, address.maxNetworkPrefixLength)
    }

    init(string: String) throws {
        let separatorIndex = string.lastIndex(of: "/") ?? string.endIndex
        let prefixStartIndex = string.index(separatorIndex, offsetBy: 1, limitedBy: string.endIndex)

        let prefixSubstring = prefixStartIndex.flatMap { string[$0...] }
        var prefix: UInt8?
        if let prefixSubstring = prefixSubstring {
            if let parsedPrefix = UInt8(prefixSubstring) {
                prefix = parsedPrefix
            } else {
                throw IPAddressRangeParseError.parsePrefix(String(prefixSubstring))
            }
        }

        let addressString = String(string[..<separatorIndex])
        if let ipv4Address = IPv4Address(addressString) {
            self = IPAddressRange(
                address: ipv4Address,
                networkPrefixLength: prefix ?? ipv4Address.maxNetworkPrefixLength
            )
        } else if let ipv6Address = IPv6Address(addressString) {
            self = IPAddressRange(
                address: ipv6Address,
                networkPrefixLength: prefix ?? ipv6Address.maxNetworkPrefixLength
            )
        } else {
            throw IPAddressRangeParseError.parseAddress(addressString)
        }
    }
}

extension IPAddressRange: Equatable {
    static func == (lhs: IPAddressRange, rhs: IPAddressRange) -> Bool {
        return lhs.address.rawValue == rhs.address.rawValue &&
            lhs.networkPrefixLength == rhs.networkPrefixLength
    }
}

extension IPAddressRange: Hashable {
    func hash(into hasher: inout Hasher) {
        hasher.combine(address.rawValue)
        hasher.combine(networkPrefixLength)
    }
}

extension IPAddressRange: CustomStringConvertible {
    var description: String {
        return "\(address)/\(networkPrefixLength)"
    }
}

private extension IPv4Address {
    var maxNetworkPrefixLength: UInt8 {
        return 32
    }
}

private extension IPv6Address {
    var maxNetworkPrefixLength: UInt8 {
        return 128
    }
}

private extension IPAddress {
    var maxNetworkPrefixLength: UInt8 {
        if let ipv4Address = self as? IPv4Address {
            return ipv4Address.maxNetworkPrefixLength
        } else if let ipv6Address = self as? IPv6Address {
            return ipv6Address.maxNetworkPrefixLength
        } else {
            fatalError()
        }
    }
}

extension IPAddressRange: Codable {
    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()

        try container.encode("\(self)")
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let value = try container.decode(String.self)

        do {
            self = try IPAddressRange(string: value)
        } catch {
            let context = DecodingError.Context(
                codingPath: container.codingPath,
                debugDescription: "Invalid IPAddressRange representation",
                underlyingError: error)
            throw DecodingError.dataCorrupted(context)
        }
    }
}

enum IPAddressRangeParseError: LocalizedError, Equatable {
    /// A failure to parse the IP address
    case parseAddress(String)

    /// A failure to parse the network prefix
    case parsePrefix(String)

    var errorDescription: String? {
        switch self {
        case .parseAddress(let addressString):
            return "Failure to parse the IP address: \(addressString)"
        case .parsePrefix(let prefixString):
            return "Failure to parse the network prefix: \(prefixString)"
        }
    }
}
