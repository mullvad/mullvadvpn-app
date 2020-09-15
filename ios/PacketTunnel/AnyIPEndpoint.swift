//
//  AnyIPEndpoint.swift
//  MullvadVPN
//
//  Created by pronebird on 14/09/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network

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

extension AnyIPEndpoint: CustomStringConvertible {
    var description: String {
        switch self {
        case .ipv4(let ipv4Endpoint):
            return "\(ipv4Endpoint)"
        case .ipv6(let ipv6Endpoint):
            return "\(ipv6Endpoint)"
        }
    }
}
