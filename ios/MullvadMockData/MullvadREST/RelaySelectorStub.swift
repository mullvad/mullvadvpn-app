//
//  RelaySelectorStub.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 05/09/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadSettings
import MullvadTypes
import Network
import WireGuardKitTypes

/// Relay selector stub that accepts a block that can be used to provide custom implementation.
public final class RelaySelectorStub: RelaySelectorProtocol {
    public let relayCache: any RelayCacheProtocol

    var selectedRelaysResult: (UInt) throws -> SelectedRelays
    var candidatesResult: (() throws -> RelayCandidates)?

    init(
        relayCache: RelayCacheProtocol = MockRelayCache(),
        selectedRelaysResult: @escaping (UInt) throws -> SelectedRelays,
        candidatesResult: (() throws -> RelayCandidates)? = nil
    ) {
        self.relayCache = relayCache
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
    ) throws -> RelayCandidates {
        return try candidatesResult?() ?? RelayCandidates(entryRelays: [], exitRelays: [])
    }

    private static let relay = SelectedRelay(
        endpoint: SelectedEndpoint(
            socketAddress: .ipv4(IPv4Endpoint(ip: .loopback, port: 1300)),
            ipv4Gateway: .loopback,
            ipv6Gateway: .loopback,
            publicKey: PrivateKey().publicKey.rawValue,
            obfuscation: .off
        ),
        hostname: "se-got",
        location: Location(
            country: "",
            countryCode: "se",
            city: "",
            cityCode: "got",
            latitude: 0,
            longitude: 0
        ),
        isOverridden: false,
        features: nil
    )

    public static let selectedRelays = SelectedRelays(
        entry: relay,
        exit: relay,
        retryAttempt: 0
    )
}

extension RelaySelectorStub {
    /// Returns a relay selector that never fails.
    public static func nonFallible() -> RelaySelectorStub {
        return RelaySelectorStub(
            selectedRelaysResult: { _ in
                return Self.selectedRelays
            }, candidatesResult: nil)
    }

    /// Returns a relay selector that cannot satisfy constraints .
    public static func unsatisfied() -> RelaySelectorStub {
        return RelaySelectorStub(
            selectedRelaysResult: { _ in
                throw NoRelaysSatisfyingConstraintsError(.relayConstraintNotMatching)
            },
            candidatesResult: {
                throw NoRelaysSatisfyingConstraintsError(.relayConstraintNotMatching)
            })
    }

}
