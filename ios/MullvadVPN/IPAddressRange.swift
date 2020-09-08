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

struct IPAddressRange {
    let address: IPAddress
    var networkPrefixLength: UInt8
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

extension IPAddressRange {

    init?(from string: String) {
        guard let parsed = IPAddressRange.parseAddressString(string) else { return nil }
        address = parsed.0
        networkPrefixLength = parsed.1
    }

    private static func parseAddressString(_ string: String) -> (IPAddress, UInt8)? {
        let endOfIPAddress = string.lastIndex(of: "/") ?? string.endIndex
        let addressString = String(string[string.startIndex ..< endOfIPAddress])
        let address: IPAddress
        if let addr = IPv4Address(addressString) {
            address = addr
        } else if let addr = IPv6Address(addressString) {
            address = addr
        } else {
            return nil
        }

        let maxNetworkPrefixLength: UInt8 = address is IPv4Address ? 32 : 128
        var networkPrefixLength: UInt8
        if endOfIPAddress < string.endIndex { // "/" was located
            let indexOfNetworkPrefixLength = string.index(after: endOfIPAddress)
            guard indexOfNetworkPrefixLength < string.endIndex else { return nil }
            let networkPrefixLengthSubstring = string[indexOfNetworkPrefixLength ..< string.endIndex]
            guard let npl = UInt8(networkPrefixLengthSubstring) else { return nil }
            networkPrefixLength = min(npl, maxNetworkPrefixLength)
        } else {
            networkPrefixLength = maxNetworkPrefixLength
        }

        return (address, networkPrefixLength)
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

        if let addressRange = IPAddressRange(from: value) {
            self = addressRange
        } else {
            throw DecodingError.dataCorruptedError(
                in: container, debugDescription: "Invalid IPAddressRange representation")
        }
    }
}
