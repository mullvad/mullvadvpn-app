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
}

extension RelaySelectorStub {
    /// Returns a relay selector that never fails.
    public static func nonFallible() -> RelaySelectorStub {
        let publicKey = PrivateKey().publicKey.rawValue

        return RelaySelectorStub(
            selectedRelaysResult: { _ in
                let cityRelay = SelectedRelay(
                    endpoint: SelectedEndpoint(
                        socketAddress: .ipv4(IPv4Endpoint(ip: .loopback, port: 1300)),
                        ipv4Gateway: .loopback,
                        ipv6Gateway: .loopback,
                        publicKey: publicKey,
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
                    features: nil
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
        return RelaySelectorStub(
            selectedRelaysResult: { _ in
                throw NoRelaysSatisfyingConstraintsError(.relayConstraintNotMatching)
            },
            candidatesResult: {
                throw NoRelaysSatisfyingConstraintsError(.relayConstraintNotMatching)
            })
    }

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
