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

public struct IPv6Endpoint: Hashable, Equatable, Codable, CustomStringConvertible, Sendable {
    public let ip: IPv6Address
    public let port: UInt16

    public init(ip: IPv6Address, port: UInt16) {
        self.ip = ip
        self.port = port
    }

    public init?(string: some StringProtocol) {
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
        "[\(ip)]:\(port)"
    }

    public static func == (lhs: IPv6Endpoint, rhs: IPv6Endpoint) -> Bool {
        lhs.ip.rawValue == rhs.ip.rawValue && lhs.port == rhs.port
    }
}
