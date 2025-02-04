//
//  SelectedRelaysStub+Stubs.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-18.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadTypes
import Network

public struct SelectedRelaysStub {
    public static let selectedRelays = SelectedRelays(
        entry: nil,
        exit: SelectedRelay(
            endpoint: MullvadEndpoint(
                ipv4Relay: IPv4Endpoint(ip: .loopback, port: 42),
                ipv6Relay: IPv6Endpoint(ip: .loopback, port: 42),
                ipv4Gateway: IPv4Address.loopback,
                ipv6Gateway: IPv6Address.loopback,
                publicKey: Data()
            ),
            hostname: "se-got-wg-001",
            location: Location(
                country: "Sweden",
                countryCode: "se",
                city: "Gothenburg",
                cityCode: "got",
                latitude: 42,
                longitude: 42
            )
        ),
        retryAttempt: 0
    )
}
