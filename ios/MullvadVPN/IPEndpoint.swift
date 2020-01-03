//
//  IPEndpoint.swift
//  MullvadVPN
//
//  Created by pronebird on 06/12/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation
import Network

/// An abstract struct describing IP address based endpoint
struct IPEndpont<T>: Hashable where T: IPAddress & Hashable {
    let ip: T
    let port: UInt16
}

extension IPEndpont: Codable where T: Codable {}

extension IPEndpont: Equatable {
    static func == (lhs: IPEndpont<T>, rhs: IPEndpont<T>) -> Bool {
        lhs.ip.rawValue == rhs.ip.rawValue && lhs.port == rhs.port
    }
}

extension IPEndpont: CustomStringConvertible {
    var description: String {
        return "\(ip):\(port)"
    }
}

/// An alias for IPv4 endpoint
typealias IPv4Endpoint = IPEndpont<IPv4Address>

/// An alias for IPv6 endpoint
typealias IPv6Endpoint = IPEndpont<IPv6Address>

/// A enum describing any IP endpoint
enum AnyIPEndpoint: Hashable {
    case ipv4(IPv4Endpoint)
    case ipv6(IPv6Endpoint)
}

extension AnyIPEndpoint: Equatable {
    static func == (lhs: AnyIPEndpoint, rhs: AnyIPEndpoint) -> Bool {
        switch (lhs, rhs) {
        case (.ipv4(let a), .ipv4(let b)):
            return a == b
        case (.ipv6(let a), .ipv6(let b)):
            return a == b
        case (.ipv6, .ipv4), (.ipv4, .ipv6):
            return false
        }
    }
}

/// Convenience methods for accessing endpoint fields
extension AnyIPEndpoint {
    var ip: IPAddress {
        switch self {
        case .ipv4(let ipv4Endpoint):
            return ipv4Endpoint.ip
        case .ipv6(let ipv6Endpoint):
            return ipv6Endpoint.ip
        }
    }

    var port: UInt16 {
        switch self {
        case .ipv4(let ipv4Endpoint):
            return ipv4Endpoint.port
        case .ipv6(let ipv6Endpoint):
            return ipv6Endpoint.port
        }
    }
}
