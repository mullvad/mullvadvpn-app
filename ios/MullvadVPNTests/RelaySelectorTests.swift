//
//  RelaySelectorTests.swift
//  RelaySelectorTests
//
//  Created by pronebird on 07/11/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadTypes
import Network
import RelaySelector
import XCTest

private let portRanges: [[UInt16]] = [[4000, 4001], [5000, 5001]]
private let defaultPort: UInt16 = 53

class RelaySelectorTests: XCTestCase {
    func testCountryConstraint() throws {
        let constraints = RelayConstraints(location: .only(.country("es")))

        let result = try RelaySelector.evaluate(
            relays: sampleRelays,
            constraints: constraints,
            numberOfFailedAttempts: 0
        )

        XCTAssertEqual(result.relay.hostname, "es1-wireguard")
    }

    func testCityConstraint() throws {
        let constraints = RelayConstraints(location: .only(.city("se", "got")))
        let result = try RelaySelector.evaluate(
            relays: sampleRelays,
            constraints: constraints,
            numberOfFailedAttempts: 0
        )

        XCTAssertEqual(result.relay.hostname, "se10-wireguard")
    }

    func testHostnameConstraint() throws {
        let constraints = RelayConstraints(location: .only(.hostname("se", "sto", "se6-wireguard")))

        let result = try RelaySelector.evaluate(
            relays: sampleRelays,
            constraints: constraints,
            numberOfFailedAttempts: 0
        )

        XCTAssertEqual(result.relay.hostname, "se6-wireguard")
    }

    func testRandomPortSelectionWithFailedAttempts() throws {
        let constraints = RelayConstraints(location: .only(.hostname("se", "sto", "se6-wireguard")))
        let allPorts = portRanges.flatMap { $0 }

        var result = try RelaySelector.evaluate(
            relays: sampleRelays,
            constraints: constraints,
            numberOfFailedAttempts: 0
        )
        XCTAssertTrue(allPorts.contains(result.endpoint.ipv4Relay.port))

        result = try RelaySelector.evaluate(relays: sampleRelays, constraints: constraints, numberOfFailedAttempts: 1)
        XCTAssertTrue(allPorts.contains(result.endpoint.ipv4Relay.port))

        result = try RelaySelector.evaluate(relays: sampleRelays, constraints: constraints, numberOfFailedAttempts: 2)
        XCTAssertEqual(result.endpoint.ipv4Relay.port, defaultPort)

        result = try RelaySelector.evaluate(relays: sampleRelays, constraints: constraints, numberOfFailedAttempts: 3)
        XCTAssertEqual(result.endpoint.ipv4Relay.port, defaultPort)

        result = try RelaySelector.evaluate(relays: sampleRelays, constraints: constraints, numberOfFailedAttempts: 4)
        XCTAssertTrue(allPorts.contains(result.endpoint.ipv4Relay.port))
    }
}

private let sampleRelays = REST.ServerRelaysResponse(
    locations: [
        "es-mad": REST.ServerLocation(
            country: "Spain",
            city: "Madrid",
            latitude: 40.408566,
            longitude: -3.69222
        ),
        "se-got": REST.ServerLocation(
            country: "Sweden",
            city: "Gothenburg",
            latitude: 57.70887,
            longitude: 11.97456
        ),
        "se-sto": REST.ServerLocation(
            country: "Sweden",
            city: "Stockholm",
            latitude: 59.3289,
            longitude: 18.0649
        ),
    ],
    wireguard: REST.ServerWireguardTunnels(
        ipv4Gateway: .loopback,
        ipv6Gateway: .loopback,
        portRanges: portRanges,
        relays: [
            REST.ServerRelay(
                hostname: "es1-wireguard",
                active: true,
                owned: true,
                location: "es-mad",
                provider: "",
                weight: 500,
                ipv4AddrIn: .loopback,
                ipv6AddrIn: .loopback,
                publicKey: Data(),
                includeInCountry: true
            ),
            REST.ServerRelay(
                hostname: "se10-wireguard",
                active: true,
                owned: true,
                location: "se-got",
                provider: "",
                weight: 1000,
                ipv4AddrIn: .loopback,
                ipv6AddrIn: .loopback,
                publicKey: Data(),
                includeInCountry: true
            ),
            REST.ServerRelay(
                hostname: "se2-wireguard",
                active: true,
                owned: true,
                location: "se-sto",
                provider: "",
                weight: 50,
                ipv4AddrIn: .loopback,
                ipv6AddrIn: .loopback,
                publicKey: Data(),
                includeInCountry: true
            ),
            REST.ServerRelay(
                hostname: "se6-wireguard",
                active: true,
                owned: true,
                location: "se-sto",
                provider: "",
                weight: 100,
                ipv4AddrIn: .loopback,
                ipv6AddrIn: .loopback,
                publicKey: Data(),
                includeInCountry: true
            ),
        ]
    ),
    bridge: REST.ServerBridges(shadowsocks: [])
)
