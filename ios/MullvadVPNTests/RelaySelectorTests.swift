//
//  RelaySelectorTests.swift
//  RelaySelectorTests
//
//  Created by pronebird on 07/11/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadREST
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

    func testSpecificPortConstraint() throws {
        let constraints = RelayConstraints(location: .only(.hostname("se", "sto", "se6-wireguard")), port: .only(1))

        let result = try RelaySelector.evaluate(
            relays: sampleRelays,
            constraints: constraints,
            numberOfFailedAttempts: 0
        )

        XCTAssertEqual(result.endpoint.ipv4Relay.port, 1)
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

    func testClosestShadowsocksRelay() throws {
        let constraints = RelayConstraints(location: .only(.city("se", "sto")))

        let selectedRelay = RelaySelector.closestShadowsocksRelayConstrained(by: constraints, in: sampleRelays)

        XCTAssertEqual(selectedRelay?.hostname, "se-sto-br-001")
    }

    func testClosestShadowsocksRelayIsRandomWhenNoContraintsAreSatisfied() throws {
        let constraints = RelayConstraints(location: .only(.country("INVALID COUNTRY")))

        let selectedRelay = try XCTUnwrap(RelaySelector.closestShadowsocksRelayConstrained(
            by: constraints,
            in: sampleRelays
        ))

        XCTAssertTrue(sampleRelays.bridge.relays.contains(selectedRelay))
    }

    func testRelayFilterConstraintWithOwnedOwnership() throws {
        let filter = RelayFilter(ownership: .owned, providers: .any)
        let constraints = RelayConstraints(
            location: .only(.hostname("se", "sto", "se6-wireguard")),
            filter: .only(filter)
        )

        let result = try RelaySelector.evaluate(
            relays: sampleRelays,
            constraints: constraints,
            numberOfFailedAttempts: 0
        )

        XCTAssertTrue(result.relay.owned)
    }

    func testRelayFilterConstraintWithRentedOwnership() throws {
        let filter = RelayFilter(ownership: .rented, providers: .any)
        let constraints = RelayConstraints(
            location: .only(.hostname("se", "sto", "se6-wireguard")),
            filter: .only(filter)
        )

        let result = try? RelaySelector.evaluate(
            relays: sampleRelays,
            constraints: constraints,
            numberOfFailedAttempts: 0
        )

        XCTAssertNil(result)
    }

    func testRelayFilterConstraintWithCorrectProvider() throws {
        let provider = "31173"

        let filter = RelayFilter(ownership: .any, providers: .only([provider]))
        let constraints = RelayConstraints(
            location: .only(.hostname("se", "sto", "se6-wireguard")),
            filter: .only(filter)
        )

        let result = try RelaySelector.evaluate(
            relays: sampleRelays,
            constraints: constraints,
            numberOfFailedAttempts: 0
        )

        XCTAssertEqual(result.relay.provider, provider)
    }

    func testRelayFilterConstraintWithIncorrectProvider() throws {
        let provider = "DataPacket"

        let filter = RelayFilter(ownership: .any, providers: .only([provider]))
        let constraints = RelayConstraints(
            location: .only(.hostname("se", "sto", "se6-wireguard")),
            filter: .only(filter)
        )

        let result = try? RelaySelector.evaluate(
            relays: sampleRelays,
            constraints: constraints,
            numberOfFailedAttempts: 0
        )

        XCTAssertNil(result)
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
        "ae-dxb": REST.ServerLocation(
            country: "United Arab Emirates",
            city: "Dubai",
            latitude: 25.276987,
            longitude: 55.296249
        ),
        "jp-tyo": REST.ServerLocation(
            country: "Japan",
            city: "Tokyo",
            latitude: 35.685,
            longitude: 139.751389
        ),
        "ca-tor": REST.ServerLocation(
            country: "Canada",
            city: "Toronto",
            latitude: 43.666667,
            longitude: -79.416667
        ),
        "us-atl": REST.ServerLocation(
            country: "USA",
            city: "Atlanta, GA",
            latitude: 40.73061,
            longitude: -73.935242
        ),
        "us-dal": REST.ServerLocation(
            country: "USA",
            city: "Dallas, TX",
            latitude: 32.89748,
            longitude: -97.040443
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
                provider: "DataPacket",
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
                provider: "31173",
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
                provider: "31173",
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
                provider: "31173",
                weight: 100,
                ipv4AddrIn: .loopback,
                ipv6AddrIn: .loopback,
                publicKey: Data(),
                includeInCountry: true
            ),
            REST.ServerRelay(
                hostname: "us-dal-wg-001",
                active: true,
                owned: true,
                location: "us-dal",
                provider: "",
                weight: 100,
                ipv4AddrIn: .loopback,
                ipv6AddrIn: .loopback,
                publicKey: Data(),
                includeInCountry: true
            ),
            REST.ServerRelay(
                hostname: "us-nyc-wg-301",
                active: true,
                owned: true,
                location: "us-nyc",
                provider: "",
                weight: 100,
                ipv4AddrIn: .loopback,
                ipv6AddrIn: .loopback,
                publicKey: Data(),
                includeInCountry: true
            ),
        ]
    ),
    bridge: REST.ServerBridges(shadowsocks: [
        REST.ServerShadowsocks(protocol: "tcp", port: 443, cipher: "aes-256-gcm", password: "mullvad"),
    ], relays: [
        REST.BridgeRelay(
            hostname: "se-sto-br-001",
            active: true,
            owned: true,
            location: "se-sto",
            provider: "31173",
            ipv4AddrIn: .loopback,
            weight: 100,
            includeInCountry: true
        ),
        REST.BridgeRelay(
            hostname: "jp-tyo-br-101",
            active: true,
            owned: true,
            location: "jp-tyo",
            provider: "M247",
            ipv4AddrIn: .loopback,
            weight: 1,
            includeInCountry: true
        ),
        REST.BridgeRelay(
            hostname: "ca-tor-ovpn-001",
            active: false,
            owned: false,
            location: "ca-tor",
            provider: "M247",
            ipv4AddrIn: .loopback,
            weight: 1,
            includeInCountry: true
        ),
        REST.BridgeRelay(
            hostname: "ae-dxb-ovpn-001",
            active: true,
            owned: false,
            location: "ae-dxb",
            provider: "M247",
            ipv4AddrIn: .loopback,
            weight: 100,
            includeInCountry: true
        ),
        REST.BridgeRelay(
            hostname: "us-atl-br-101",
            active: true,
            owned: false,
            location: "us-atl",
            provider: "100TB",
            ipv4AddrIn: .loopback,
            weight: 100,
            includeInCountry: true
        ),
        REST.BridgeRelay(
            hostname: "us-dal-br-101",
            active: true,
            owned: false,
            location: "us-dal",
            provider: "100TB",
            ipv4AddrIn: .loopback,
            weight: 100,
            includeInCountry: true
        ),
    ])
)
