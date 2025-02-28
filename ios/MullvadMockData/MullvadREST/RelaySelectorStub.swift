//
//  RelaySelectorStub.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 05/09/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadSettings
import MullvadTypes
import WireGuardKitTypes

/// Relay selector stub that accepts a block that can be used to provide custom implementation.
public final class RelaySelectorStub: RelaySelectorProtocol {
    var selectedRelaysResult: (UInt) throws -> SelectedRelays
    var candidatesResult: (() throws -> RelaysCandidates)?

    init(
        selectedRelaysResult: @escaping (UInt) throws -> SelectedRelays,
        candidatesResult: (() throws -> RelaysCandidates)? = nil
    ) {
        self.selectedRelaysResult = selectedRelaysResult
        self.candidatesResult = candidatesResult
    }

    public func selectRelays(
        tunnelSettings: LatestTunnelSettings,
        connectionAttemptCount: UInt
    ) throws -> SelectedRelays {
        return try selectedRelaysResult(connectionAttemptCount)
    }

    public func findCandidates(
        tunnelSettings: LatestTunnelSettings
    ) throws -> RelaysCandidates {
        return try candidatesResult?() ?? RelaysCandidates(entryRelays: [], exitRelays: [])
    }
}

extension RelaySelectorStub {
    /// Returns a relay selector that never fails.
    public static func nonFallible() -> RelaySelectorStub {
        let publicKey = PrivateKey().publicKey.rawValue

        return RelaySelectorStub(selectedRelaysResult: { _ in
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
        }, candidatesResult: nil)
    }

    /// Returns a relay selector that cannot satisfy constraints .
    public static func unsatisfied() -> RelaySelectorStub {
        return RelaySelectorStub(selectedRelaysResult: { _ in
            throw NoRelaysSatisfyingConstraintsError(.relayConstraintNotMatching)
        }, candidatesResult: {
            throw NoRelaysSatisfyingConstraintsError(.relayConstraintNotMatching)
        })
    }
}
