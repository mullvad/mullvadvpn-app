//
//  IPEndpoint.swift
//  MullvadVPN
//
//  Created by pronebird on 06/12/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
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
        if ip is IPv4Address {
            return "\(ip):\(port)"
        } else if ip is IPv6Address {
            return "[\(ip)]:\(port)"
        } else {
            fatalError()
        }
    }
}

/// An alias for IPv4 endpoint
typealias IPv4Endpoint = IPEndpont<IPv4Address>

/// An alias for IPv6 endpoint
typealias IPv6Endpoint = IPEndpont<IPv6Address>
