//
//  RelaySelectorStub.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 05/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import PacketTunnelCore
import class WireGuardKitTypes.PrivateKey

/// Relay selector mock that accepts a block that can be used to provide custom implementation.
struct MockRelaySelector: RelaySelectorProtocol {
    let block: (RelayConstraints, UInt) throws -> SelectedRelay

    func selectRelay(
        with constraints: RelayConstraints,
        connectionAttemptFailureCount: UInt
    ) throws -> SelectedRelay {
        return try block(constraints, connectionAttemptFailureCount)
    }
}

extension RelaySelectorStub {
    /// Returns a relay selector that never fails.
    static func nonFallible() -> RelaySelectorStub {
        let publicKey = PrivateKey().publicKey.rawValue

        return MockRelaySelector { _, _ in
            return SelectedRelay(
                endpoint: MullvadEndpoint(
                    ipv4Relay: IPv4Endpoint(ip: .loopback, port: 1300),
                    ipv4Gateway: .loopback,
                    ipv6Gateway: .loopback,
                    publicKey: publicKey
                ),
                hostname: "se-got",
                location: Location(
                    country: "",
                    countryCode: "se",
                    city: "",
                    cityCode: "got",
                    latitude: 0,
                    longitude: 0
                )
            )
        }
    }
}
