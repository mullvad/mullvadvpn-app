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

/// A fully-resolved endpoint with a single socket address and bundled obfuscation info.
///
/// This type represents the output of relay selection, containing a resolved IP address
/// (either IPv4 or IPv6, based on user preference) along with the obfuscation method.
public struct SelectedEndpoint: Equatable, Codable, Sendable {
    /// The socket address to connect to (either IPv4 or IPv6).
    public let socketAddress: AnyIPEndpoint

    /// The IPv4 gateway address for DNS resolution.
    public let ipv4Gateway: IPv4Address

    /// The IPv6 gateway address for DNS resolution.
    public let ipv6Gateway: IPv6Address

    /// The relay's WireGuard public key.
    public let publicKey: Data

    /// The obfuscation method in use, bundled with the endpoint.
    public let obfuscation: ObfuscationMethod

    public init(
        socketAddress: AnyIPEndpoint,
        ipv4Gateway: IPv4Address,
        ipv6Gateway: IPv6Address,
        publicKey: Data,
        obfuscation: ObfuscationMethod
    ) {
        self.socketAddress = socketAddress
        self.ipv4Gateway = ipv4Gateway
        self.ipv6Gateway = ipv6Gateway
        self.publicKey = publicKey
        self.obfuscation = obfuscation
    }
}

extension SelectedEndpoint: CustomDebugStringConvertible {
    public var debugDescription: String {
        "\(socketAddress) (obfuscation: \(obfuscation))"
    }
}
