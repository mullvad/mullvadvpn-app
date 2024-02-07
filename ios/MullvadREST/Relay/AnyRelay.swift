//
//  AnyRelay.swift
//  MullvadREST
//
//  Created by Jon Petersson on 2024-01-31.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
import Network

public protocol AnyRelay {
    var hostname: String { get }
    var owned: Bool { get }
    var location: String { get }
    var provider: String { get }
    var weight: UInt64 { get }
    var active: Bool { get }
    var includeInCountry: Bool { get }

    func override(ipv4AddrIn: IPv4Address?, ipv6AddrIn: IPv6Address?) -> Self
}

extension REST.ServerRelay: AnyRelay {}
extension REST.BridgeRelay: AnyRelay {
    public func override(ipv4AddrIn: IPv4Address?, ipv6AddrIn: IPv6Address?) -> REST.BridgeRelay {
        override(ipv4AddrIn: ipv4AddrIn)
    }
}
