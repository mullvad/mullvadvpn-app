//
//  AnyIPEndpoint+Socks5.swift
//  MullvadTransport
//
//  Created by pronebird on 23/10/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import Network

extension AnyIPEndpoint {
    /// Convert `AnyIPEndpoint` to `Socks5Endpoint`.
    var socksEndpoint: Socks5Endpoint {
        switch self {
        case let .ipv4(endpoint):
            .ipv4(endpoint)
        case let .ipv6(endpoint):
            .ipv6(endpoint)
        }
    }

    /// Convert `AnyIPEndpoint` to `NWEndpoint`.
    var nwEndpoint: NWEndpoint {
        switch self {
        case let .ipv4(endpoint):
            .hostPort(host: .ipv4(endpoint.ip), port: NWEndpoint.Port(integerLiteral: endpoint.port))
        case let .ipv6(endpoint):
            .hostPort(host: .ipv6(endpoint.ip), port: NWEndpoint.Port(integerLiteral: endpoint.port))
        }
    }
}
