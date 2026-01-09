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
            endpoint: SelectedEndpoint(
                socketAddress: .ipv4(IPv4Endpoint(ip: .loopback, port: 42)),
                ipv4Gateway: IPv4Address.loopback,
                ipv6Gateway: IPv6Address.loopback,
                publicKey: Data(),
                obfuscation: .off
            ),
            hostname: "se-got-wg-001",
            location: Location(
                country: "Sweden",
                countryCode: "se",
                city: "Gothenburg",
                cityCode: "got",
                latitude: 42,
                longitude: 42
            ),
            features: nil
        ),
        retryAttempt: 0
    )
}
