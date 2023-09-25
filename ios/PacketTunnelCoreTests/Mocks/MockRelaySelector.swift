//
//  MockRelaySelector.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 05/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
@testable import MullvadREST
import MullvadTypes
import PacketTunnelCore
@testable import RelaySelector
import class WireGuardKitTypes.PrivateKey

/// Relay selector mock that accepts a block that can be used to provide custom implementation.
struct MockRelaySelector: RelaySelectorProtocol {
    let block: (RelayConstraints, UInt) throws -> RelaySelectorResult

    func selectRelay(
        with constraints: RelayConstraints,
        connectionAttemptFailureCount: UInt
    ) throws -> RelaySelectorResult {
        return try block(constraints, connectionAttemptFailureCount)
    }
}

extension MockRelaySelector {
    /// Returns a relay selector that never fails.
    static func nonFallible() -> MockRelaySelector {
        let publicKey = PrivateKey().publicKey.rawValue

        return MockRelaySelector { _, _ in
            return RelaySelectorResult(
                endpoint: MullvadEndpoint(
                    ipv4Relay: IPv4Endpoint(ip: .loopback, port: 1300),
                    ipv4Gateway: .loopback,
                    ipv6Gateway: .loopback,
                    publicKey: publicKey
                ),
                relay: REST.ServerRelay(
                    hostname: "se-got",
                    active: true,
                    owned: true,
                    location: "se-got",
                    provider: "",
                    weight: 0,
                    ipv4AddrIn: .loopback,
                    ipv6AddrIn: .loopback,
                    publicKey: publicKey,
                    includeInCountry: true
                ),
                location: Location(country: "", countryCode: "se", city: "", cityCode: "got", latitude: 0, longitude: 0)
            )
        }
    }
}
