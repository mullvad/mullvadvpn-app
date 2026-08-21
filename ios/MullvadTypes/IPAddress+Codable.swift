// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Network

extension IPv4Address: @retroactive Decodable {}
extension IPv4Address: @retroactive Encodable {}

extension IPv4Address {
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

extension IPv6Address: @retroactive Decodable {}
extension IPv6Address: @retroactive Encodable {}

extension IPv6Address {
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
