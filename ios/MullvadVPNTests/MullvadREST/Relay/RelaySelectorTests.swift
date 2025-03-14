//
//  RelaySelectorTests.swift
//  RelaySelectorTests
//
//  Created by pronebird on 07/11/2019.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadMockData
@testable import MullvadREST
@testable import MullvadSettings
import MullvadTypes
import Network
@testable import WireGuardKitTypes
import XCTest

private let portRanges: [[UInt16]] = [[4000, 4001], [5000, 5001]]
private let defaultPort: UInt16 = 443

class RelaySelectorTests: XCTestCase {
    let sampleRelays = ServerRelaysResponseStubs.sampleRelays

    func testCountryConstraint() throws {
        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.country("es")]))
        )

        let result = try pickRelay(by: constraints, in: sampleRelays, failedAttemptCount: 0)
        XCTAssertEqual(result.relay.hostname, "es1-wireguard")
    }

    func testCityConstraint() throws {
        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.city("se", "got")]))
        )

        let result = try pickRelay(by: constraints, in: sampleRelays, failedAttemptCount: 0)
        XCTAssertEqual(result.relay.hostname, "se10-wireguard")
    }

    func testHostnameConstraint() throws {
        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "sto", "se6-wireguard")]))
        )

        let result = try pickRelay(by: constraints, in: sampleRelays, failedAttemptCount: 0)
        XCTAssertEqual(result.relay.hostname, "se6-wireguard")
    }

    func testMultipleLocationsConstraint() throws {
        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [
                .city("se", "got"),
                .hostname("se", "sto", "se6-wireguard"),
            ]))
        )

        let relayWithLocations = sampleRelays.wireguard.relays.map {
            let location = sampleRelays.locations[$0.location.rawValue]!

            return RelayWithLocation(
                relay: $0,
                serverLocation: Location(
                    country: location.country,
                    countryCode: String($0.location.country),
                    city: location.city,
                    cityCode: String($0.location.city),
                    latitude: location.latitude,
                    longitude: location.longitude
                )
            )
        }

        let constrainedLocations = try RelaySelector.applyConstraints(
            constraints.exitLocations,
            filterConstraint: constraints.filter,
            daitaEnabled: false,
            relays: relayWithLocations
        )

        XCTAssertTrue(
            constrainedLocations.contains(
                where: { $0.matches(location: .city("se", "got")) }
            )
        )

        XCTAssertTrue(
            constrainedLocations.contains(
                where: { $0.matches(location: .hostname("se", "sto", "se6-wireguard")) }
            )
        )
    }

    func testNoMatchingRelayConstraint() throws {
        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.country("-")]))
        )

        XCTAssertThrowsError(
            try pickRelay(by: constraints, in: sampleRelays, failedAttemptCount: 0)
        ) { error in
            let error = error as? NoRelaysSatisfyingConstraintsError
            XCTAssertEqual(error?.reason, .relayConstraintNotMatching)
        }
    }

    func testSpecificPortConstraint() throws {
        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "sto", "se6-wireguard")])),
            port: .only(1)
        )

        let result = try pickRelay(by: constraints, in: sampleRelays, failedAttemptCount: 0)
        XCTAssertEqual(result.endpoint.ipv4Relay.port, 1)
    }

    func testRandomPortSelectionWithFailedAttempts() throws {
        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "sto", "se6-wireguard")]))
        )
        let allPorts = portRanges.flatMap { $0 }

        var result = try pickRelay(by: constraints, in: sampleRelays, failedAttemptCount: 0)
        XCTAssertTrue(allPorts.contains(result.endpoint.ipv4Relay.port))

        result = try pickRelay(by: constraints, in: sampleRelays, failedAttemptCount: 1)
        XCTAssertEqual(result.endpoint.ipv4Relay.port, defaultPort)

        result = try pickRelay(by: constraints, in: sampleRelays, failedAttemptCount: 2)
        XCTAssertTrue(allPorts.contains(result.endpoint.ipv4Relay.port))

        result = try pickRelay(by: constraints, in: sampleRelays, failedAttemptCount: 3)
        XCTAssertEqual(result.endpoint.ipv4Relay.port, defaultPort)

        result = try pickRelay(by: constraints, in: sampleRelays, failedAttemptCount: 4)
        XCTAssertTrue(allPorts.contains(result.endpoint.ipv4Relay.port))
    }

    func testClosestRelay() throws {
        let relayWithLocations = try sampleRelays.wireguard.relays.map {
            let serverLocation = try XCTUnwrap(sampleRelays.locations[$0.location.rawValue])
            let location = Location(
                country: serverLocation.country,
                countryCode: serverLocation.country,
                city: serverLocation.city,
                cityCode: serverLocation.city,
                latitude: serverLocation.latitude,
                longitude: serverLocation.longitude
            )

            return RelayWithLocation(relay: $0, serverLocation: location)
        }

        let sampleLocation = try XCTUnwrap(sampleRelays.locations["se-got"])
        let location = Location(
            country: "Sweden",
            countryCode: sampleLocation.country,
            city: "Gothenburg",
            cityCode: sampleLocation.city,
            latitude: sampleLocation.latitude,
            longitude: sampleLocation.longitude
        )

        let selectedRelay = RelaySelector.WireGuard.closestRelay(
            to: location,
            using: relayWithLocations
        )

        XCTAssertEqual(selectedRelay?.hostname, "se10-wireguard")
    }

    func testClosestShadowsocksRelay() throws {
        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.city("se", "sto")]))
        )

        let selectedRelay = RelaySelector.Shadowsocks.closestRelay(
            location: constraints.exitLocations,
            port: constraints.port,
            filter: constraints.filter,
            in: sampleRelays
        )

        XCTAssertEqual(selectedRelay?.hostname, "se-sto-br-001")
    }

    func testClosestShadowsocksRelayIsRandomWhenNoContraintsAreSatisfied() throws {
        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.country("INVALID COUNTRY")]))
        )

        let selectedRelay = try XCTUnwrap(RelaySelector.Shadowsocks.closestRelay(
            location: constraints.exitLocations,
            port: constraints.port,
            filter: constraints.filter,
            in: sampleRelays
        ))

        XCTAssertTrue(sampleRelays.bridge.relays.contains(selectedRelay))
    }

    func testRelayFilterConstraintWithOwnedOwnership() throws {
        let filter = RelayFilter(ownership: .owned, providers: .any)

        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "sto", "se6-wireguard")])),
            filter: .only(filter)
        )

        let result = try pickRelay(by: constraints, in: sampleRelays, failedAttemptCount: 0)
        XCTAssertTrue(result.relay.owned)
    }

    func testRelayFilterConstraintWithRentedOwnership() throws {
        let filter = RelayFilter(ownership: .rented, providers: .any)

        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("es", "mad", "es1-wireguard")])),
            filter: .only(filter)
        )

        let result = try pickRelay(by: constraints, in: sampleRelays, failedAttemptCount: 0)
        XCTAssertNotEqual(result.relay.owned, true)
    }

    func testRelayFilterConstraintWithCorrectProvider() throws {
        let provider = "31173"
        let filter = RelayFilter(ownership: .any, providers: .only([provider]))

        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "sto", "se6-wireguard")])),
            filter: .only(filter)
        )

        let result = try pickRelay(by: constraints, in: sampleRelays, failedAttemptCount: 0)
        XCTAssertEqual(result.relay.provider, provider)
    }

    func testRelayFilterConstraintWithIncorrectProvider() throws {
        let provider = ""
        let filter = RelayFilter(ownership: .any, providers: .only([provider]))

        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "sto", "se6-wireguard")])),
            filter: .only(filter)
        )

        XCTAssertThrowsError(try pickRelay(by: constraints, in: sampleRelays, failedAttemptCount: 0)) { error in
            let error = error as? NoRelaysSatisfyingConstraintsError
            XCTAssertEqual(error?.reason, .filterConstraintNotMatching)
        }
    }

    func testRelayWithDaita() throws {
        let hasDaitaConstraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.country("es")]))
        )

        let noDaitaConstraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.country("se")]))
        )

        XCTAssertNoThrow(
            try pickRelay(
                by: hasDaitaConstraints,
                in: sampleRelays,
                failedAttemptCount: 0,
                daitaEnabled: true
            )
        )
        XCTAssertThrowsError(
            try pickRelay(by: noDaitaConstraints, in: sampleRelays, failedAttemptCount: 0, daitaEnabled: true)
        ) { error in
            let error = error as? NoRelaysSatisfyingConstraintsError
            XCTAssertEqual(error?.reason, .noDaitaRelaysFound)
        }
    }

    func testNoActiveRelaysError() throws {
        XCTAssertThrowsError(
            try pickRelay(by: RelayConstraints(), in: sampleRelaysNoActive, failedAttemptCount: 0)
        ) { error in
            let error = error as? NoRelaysSatisfyingConstraintsError
            XCTAssertEqual(error?.reason, .noActiveRelaysFound)
        }
    }
}

extension RelaySelectorTests {
    private func pickRelay(
        by constraints: RelayConstraints,
        in relays: REST.ServerRelaysResponse,
        failedAttemptCount: UInt,
        daitaEnabled: Bool = false
    ) throws -> RelaySelectorMatch {
        let candidates = try RelaySelector.WireGuard.findCandidates(
            by: constraints.exitLocations,
            in: relays,
            filterConstraint: constraints.filter,
            daitaEnabled: daitaEnabled
        )

        return try RelaySelector.WireGuard.pickCandidate(
            from: candidates,
            wireguard: relays.wireguard,
            portConstraint: constraints.port,
            numberOfFailedAttempts: failedAttemptCount
        )
    }
}

extension RelaySelectorTests {
    var sampleRelaysNoActive: REST.ServerRelaysResponse {
        REST.ServerRelaysResponse(
            locations: [
                "es-mad": REST.ServerLocation(
                    country: "Spain",
                    city: "Madrid",
                    latitude: 40.408566,
                    longitude: -3.69222
                ),
            ],
            wireguard: REST.ServerWireguardTunnels(
                ipv4Gateway: .loopback,
                ipv6Gateway: .loopback,
                portRanges: portRanges,
                relays: [
                    REST.ServerRelay(
                        hostname: "es1-wireguard",
                        active: false,
                        owned: true,
                        location: "es-mad",
                        provider: "",
                        weight: 500,
                        ipv4AddrIn: .loopback,
                        ipv6AddrIn: .loopback,
                        publicKey: PrivateKey().publicKey.rawValue,
                        includeInCountry: true,
                        daita: true,
                        shadowsocksExtraAddrIn: nil
                    ),
                ],
                shadowsocksPortRanges: []
            ),
            bridge: REST.ServerBridges(shadowsocks: [], relays: [])
        )
    }
}
