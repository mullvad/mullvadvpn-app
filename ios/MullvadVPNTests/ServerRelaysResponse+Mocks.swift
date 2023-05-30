//
//  ServerRelaysResponse+Mocks.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-06-02.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import Network

extension REST.ServerRelaysResponse {
    static var empty: Self {
        REST.ServerRelaysResponse(locations: [:], wireguard: .empty, bridge: .empty)
    }
}

extension REST.ServerLocation {
    static var empty: Self {
        .init(country: "", city: "", latitude: 0, longitude: 0)
    }
}

extension REST.ServerWireguardTunnels {
    static var empty: Self {
        .init(ipv4Gateway: .loopback, ipv6Gateway: .loopback, portRanges: [], relays: [])
    }
}

extension REST.ServerBridges {
    static var empty: Self {
        .init(shadowsocks: [], relays: [])
    }
}
