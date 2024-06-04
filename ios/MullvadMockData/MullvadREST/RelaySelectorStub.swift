//
//  RelaySelectorStub.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 05/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
import MullvadREST
import WireGuardKitTypes

/// Relay selector stub that accepts a block that can be used to provide custom implementation.
public struct RelaySelectorStub: RelaySelectorProtocol {
    let block: (RelayConstraints, UInt) throws -> SelectedRelays

    public func selectRelays(
        with constraints: RelayConstraints,
        connectionAttemptCount: UInt
    ) throws -> SelectedRelays {
        return try block(constraints, connectionAttemptCount)
    }
}

extension RelaySelectorStub {
    /// Returns a relay selector that never fails.
    public static func nonFallible() -> RelaySelectorStub {
        let publicKey = PrivateKey().publicKey.rawValue

        return RelaySelectorStub { _, _ in
            let cityRelay = SelectedRelay(
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
                ), retryAttempts: 0
            )

            return SelectedRelays(
                entry: cityRelay,
                exit: cityRelay
            )
        }
    }
}
