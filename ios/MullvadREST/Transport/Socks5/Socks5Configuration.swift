//
//  Socks5Configuration.swift
//  MullvadTransport
//
//  Created by pronebird on 23/10/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import Network

/// Socks5 configuration.
/// - See: ``URLSessionSocks5Transport``
public struct Socks5Configuration {
    public let address: AnyIPAddress
    public let port: UInt16

    public init(address: AnyIPAddress, port: UInt16) {
        self.address = address
        self.port = port
    }

    var nwEndpoint: NWEndpoint {
        switch self.address {
        case let .ipv4(endpoint):
            .hostPort(host: .ipv4(endpoint), port: NWEndpoint.Port(integerLiteral: port))
        case let .ipv6(endpoint):
            .hostPort(host: .ipv6(endpoint), port: NWEndpoint.Port(integerLiteral: port))
        }
    }
}
