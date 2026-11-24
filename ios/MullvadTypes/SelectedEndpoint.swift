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

/// The addresses of a relay that a tunnel may connect to.
public enum SelectedSocketAddress: Hashable, Codable, Sendable {
    case ipv4(IPv4Endpoint)
    case ipv6(IPv6Endpoint)
    case both(IPv4Endpoint, IPv6Endpoint)

    public init(_ endpoint: AnyIPEndpoint) {
        switch endpoint {
        case let .ipv4(ipv4): self = .ipv4(ipv4)
        case let .ipv6(ipv6): self = .ipv6(ipv6)
        }
    }

    /// The address that is tried first.
    public var preferred: AnyIPEndpoint {
        switch self {
        case let .ipv4(ipv4), let .both(ipv4, _): .ipv4(ipv4)
        case let .ipv6(ipv6): .ipv6(ipv6)
        }
    }

    public var ip: IPAddress { preferred.ip }
    public var port: UInt16 { preferred.port }
}

extension SelectedSocketAddress: CustomStringConvertible {
    public var description: String {
        switch self {
        case let .ipv4(ipv4): "IPv4: \(ipv4), IPv6: NONE"
        case let .ipv6(ipv6): "IPv4: NONE, IPv6: \(ipv6)"
        case let .both(ipv4, ipv6): "IPv4: \(ipv4), IPv6: \(ipv6)"
        }
    }
}

/// A fully-resolved endpoint with its socket addresses and bundled obfuscation info.
///
/// This type represents the output of relay selection, containing the resolved IP addresses
/// (IPv4, IPv6 or both, based on user preference) along with the obfuscation method.
public struct SelectedEndpoint: Equatable, Codable, Sendable {
    /// The socket addresses to connect to.
    public let socketAddress: SelectedSocketAddress

    /// The IPv4 gateway address for DNS resolution.
    public let ipv4Gateway: IPv4Address

    /// The IPv6 gateway address for DNS resolution.
    public let ipv6Gateway: IPv6Address

    /// The relay's WireGuard public key.
    public let publicKey: Data

    /// The obfuscation method in use, bundled with the endpoint.
    public let obfuscation: ObfuscationMethod

    public init(
        socketAddress: SelectedSocketAddress,
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

    /// The endpoint with `socketAddress` as its only address.
    public func with(socketAddress: AnyIPEndpoint) -> SelectedEndpoint {
        SelectedEndpoint(
            socketAddress: SelectedSocketAddress(socketAddress),
            ipv4Gateway: ipv4Gateway,
            ipv6Gateway: ipv6Gateway,
            publicKey: publicKey,
            obfuscation: obfuscation
        )
    }
}

extension SelectedEndpoint: CustomDebugStringConvertible {
    public var debugDescription: String {
        "\(socketAddress) (obfuscation: \(obfuscation))"
    }
}
