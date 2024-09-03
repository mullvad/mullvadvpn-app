//
//  RelaySelectorStub.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 05/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadSettings
import MullvadTypes
import WireGuardKitTypes

/// Relay selector stub that accepts a block that can be used to provide custom implementation.
public final class RelaySelectorStub: RelaySelectorProtocol {
    var selectedRelaysResult: (UInt) throws -> SelectedRelays

    init(selectedRelaysResult: @escaping (UInt) throws -> SelectedRelays) {
        self.selectedRelaysResult = selectedRelaysResult
    }

    public func selectRelays(
        tunnelSettings: LatestTunnelSettings,
        connectionAttemptCount: UInt
    ) throws -> SelectedRelays {
        return try selectedRelaysResult(connectionAttemptCount)
    }
}

extension RelaySelectorStub {
    /// Returns a relay selector that never fails.
    public static func nonFallible() -> RelaySelectorStub {
        let publicKey = PrivateKey().publicKey.rawValue

        return RelaySelectorStub { _ in
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
                )
            )

            return SelectedRelays(
                entry: cityRelay,
                exit: cityRelay,
                retryAttempt: 0
            )
        }
    }

    /// Returns a relay selector that cannot satisfy constraints .
    public static func unsatisfied() -> RelaySelectorStub {
        return RelaySelectorStub { _ in
            throw NoRelaysSatisfyingConstraintsError(.relayConstraintNotMatching)
        }
    }
}
