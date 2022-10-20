//
//  AnyIPEndpoint.swift
//  MullvadTypes
//
//  Created by pronebird on 20/10/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import protocol Network.IPAddress

public enum AnyIPEndpoint: Hashable, Equatable, Codable, CustomStringConvertible {
    case ipv4(IPv4Endpoint)
    case ipv6(IPv6Endpoint)

    public var ip: IPAddress {
        switch self {
        case let .ipv4(ipv4Endpoint):
            return ipv4Endpoint.ip
        case let .ipv6(ipv6Endpoint):
            return ipv6Endpoint.ip
        }
    }

    public var port: UInt16 {
        switch self {
        case let .ipv4(ipv4Endpoint):
            return ipv4Endpoint.port
        case let .ipv6(ipv6Endpoint):
            return ipv6Endpoint.port
        }
    }

    public init?<S>(string: S) where S: StringProtocol {
        if let ipv4Endpoint = IPv4Endpoint(string: string) {
            self = .ipv4(ipv4Endpoint)
        } else if let ipv6Endpoint = IPv6Endpoint(string: string) {
            self = .ipv6(ipv6Endpoint)
        } else {
            return nil
        }
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let string = try container.decode(String.self)

        if let ipv4Endpoint = IPv4Endpoint(string: string) {
            self = .ipv4(ipv4Endpoint)
        } else if let ipv6Endpoint = IPv6Endpoint(string: string) {
            self = .ipv6(ipv6Endpoint)
        } else {
            throw DecodingError.dataCorruptedError(
                in: container,
                debugDescription: "Cannot parse the endpoint"
            )
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()

        try container.encode("\(self)")
    }

    public var description: String {
        switch self {
        case let .ipv4(ipv4Endpoint):
            return "\(ipv4Endpoint)"
        case let .ipv6(ipv6Endpoint):
            return "\(ipv6Endpoint)"
        }
    }

    public static func == (lhs: AnyIPEndpoint, rhs: AnyIPEndpoint) -> Bool {
        switch (lhs, rhs) {
        case let (.ipv4(lhsEndpoint), .ipv4(rhsEndpoint)):
            return lhsEndpoint == rhsEndpoint

        case let (.ipv6(lhsEndpoint), .ipv6(rhsEndpoint)):
            return lhsEndpoint == rhsEndpoint

        default:
            return false
        }
    }
}
