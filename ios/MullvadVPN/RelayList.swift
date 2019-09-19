
//
//  RelayList.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation
import Network

struct RelayList: Codable {
    struct Country: Codable {
        let name: String
        let code: String
        let cities: [City]
    }

    struct City: Codable {
        let name: String
        let code: String
        let latitude: Double
        let longitude: Double
        let relays: [Hostname]
    }

    struct Hostname: Codable {
        let hostname: String
        let ipv4AddrIn: IPv4Address
        let includeInCountry: Bool
        let weight: Int32
        let tunnels: Tunnels?
    }

    struct Tunnels: Codable {
        let wireguard: [WireguardTunnel]?
    }

    struct WireguardTunnel: Codable {
        let ipv4Gateway: IPv4Address
        let ipv6Gateway: IPv6Address
        let publicKey: Data
        let portRanges: [ClosedRange<UInt16>]
    }

    let countries: [Country]
}
