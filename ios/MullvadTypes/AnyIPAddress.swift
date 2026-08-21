// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import Network

/// Container type that holds either `IPv4Address` or `IPv6Address`.
public enum AnyIPAddress: IPAddress, Codable, Equatable, CustomDebugStringConvertible {
    case ipv4(IPv4Address)
    case ipv6(IPv6Address)

    private enum CodingKeys: String, CodingKey {
        case ipv4, ipv6
    }

    private var innerAddress: IPAddress {
        switch self {
        case let .ipv4(ipv4Address):
            return ipv4Address
        case let .ipv6(ipv6Address):
            return ipv6Address
        }
    }

    public var rawValue: Data {
        innerAddress.rawValue
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)

        if container.contains(.ipv4) {
            self = .ipv4(try container.decode(IPv4Address.self, forKey: .ipv4))
        } else if container.contains(.ipv6) {
            self = .ipv6(try container.decode(IPv6Address.self, forKey: .ipv6))
        } else {
            throw DecodingError.dataCorruptedError(
                forKey: .ipv4,
                in: container,
                debugDescription: "Invalid AnyIPAddress representation"
            )
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)

        switch self {
        case let .ipv4(ipv4Address):
            try container.encode(ipv4Address, forKey: .ipv4)
        case let .ipv6(ipv6Address):
            try container.encode(ipv6Address, forKey: .ipv6)
        }
    }

    public init?(_ rawValue: Data, _ interface: NWInterface?) {
        if let ipv4Address = IPv4Address(rawValue, interface) {
            self = .ipv4(ipv4Address)
        } else if let ipv6Address = IPv6Address(rawValue, interface) {
            self = .ipv6(ipv6Address)
        } else {
            return nil
        }
    }

    public init?(_ string: String) {
        // Arbitrary integers should not be allowed by us and need to be handled separately
        // since Apple allows them.
        guard Int(string) == nil else { return nil }

        if let ipv4Address = IPv4Address(string) {
            self = .ipv4(ipv4Address)
        } else if let ipv6Address = IPv6Address(string) {
            self = .ipv6(ipv6Address)
        } else {
            return nil
        }
    }

    public var interface: NWInterface? {
        innerAddress.interface
    }

    public var isLoopback: Bool {
        innerAddress.isLoopback
    }

    public var isLinkLocal: Bool {
        innerAddress.isLinkLocal
    }

    public var isMulticast: Bool {
        innerAddress.isMulticast
    }

    public var isIPv4: Bool {
        if case .ipv4 = self { return true }
        return false
    }

    public var isIPv6: Bool {
        if case .ipv6 = self { return true }
        return false
    }

    public var debugDescription: String {
        switch self {
        case let .ipv4(ipv4Address):
            return "\(ipv4Address)"
        case let .ipv6(ipv6Address):
            return "\(ipv6Address)"
        }
    }
}
