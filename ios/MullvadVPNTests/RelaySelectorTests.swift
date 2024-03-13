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
import XCTest

private let portRanges: [[UInt16]] = [[4000, 4001], [5000, 5001]]
private let defaultPort: UInt16 = 53

class RelaySelectorTests: XCTestCase {
    let sampleRelays = ServerRelaysResponseStubs.sampleRelays

    func testCountryConstraint() throws {
        let constraints = RelayConstraints(
            locations: .only(UserSelectedRelays(locations: [.country("es")]))
        )

        let result = try RelaySelector.evaluate(
            relays: sampleRelays,
            constraints: constraints,
            numberOfFailedAttempts: 0
        )

        XCTAssertEqual(result.relay.hostname, "es1-wireguard")
    }

    func testCityConstraint() throws {
        let constraints = RelayConstraints(
            locations: .only(UserSelectedRelays(locations: [.city("se", "got")]))
        )

        let result = try RelaySelector.evaluate(
            relays: sampleRelays,
            constraints: constraints,
            numberOfFailedAttempts: 0
        )

        XCTAssertEqual(result.relay.hostname, "se10-wireguard")
    }

    func testHostnameConstraint() throws {
        let constraints = RelayConstraints(
            locations: .only(UserSelectedRelays(locations: [.hostname("se", "sto", "se6-wireguard")]))
        )

        let result = try RelaySelector.evaluate(
            relays: sampleRelays,
            constraints: constraints,
            numberOfFailedAttempts: 0
        )

        XCTAssertEqual(result.relay.hostname, "se6-wireguard")
    }

    func testMultipleLocationsConstraint() throws {
        let constraints = RelayConstraints(
            locations: .only(UserSelectedRelays(locations: [
                .city("se", "got"),
                .hostname("se", "sto", "se6-wireguard"),
            ]))
        )

        let relayWithLocations = sampleRelays.wireguard.relays.map {
            let location = sampleRelays.locations[$0.location]!
            let locationComponents = $0.location.split(separator: "-")

            return RelayWithLocation(
                relay: $0,
                serverLocation: Location(
                    country: location.country,
                    countryCode: String(locationComponents[0]),
                    city: location.city,
                    cityCode: String(locationComponents[1]),
                    latitude: location.latitude,
                    longitude: location.longitude
                )
            )
        }

        let constrainedLocations = RelaySelector.applyConstraints(constraints, relays: relayWithLocations)

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

    func testSpecificPortConstraint() throws {
        let constraints = RelayConstraints(
            locations: .only(UserSelectedRelays(locations: [.hostname("se", "sto", "se6-wireguard")])),
            port: .only(1)
        )

        let result = try RelaySelector.evaluate(
            relays: sampleRelays,
            constraints: constraints,
            numberOfFailedAttempts: 0
        )

        XCTAssertEqual(result.endpoint.ipv4Relay.port, 1)
    }

    func testRandomPortSelectionWithFailedAttempts() throws {
        let constraints = RelayConstraints(
            locations: .only(UserSelectedRelays(locations: [.hostname("se", "sto", "se6-wireguard")]))
        )
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
        let constraints = RelayConstraints(
            locations: .only(UserSelectedRelays(locations: [.city("se", "sto")]))
        )

        let selectedRelay = RelaySelector.closestShadowsocksRelayConstrained(by: constraints, in: sampleRelays)

        XCTAssertEqual(selectedRelay?.hostname, "se-sto-br-001")
    }

    func testClosestShadowsocksRelayIsRandomWhenNoContraintsAreSatisfied() throws {
        let constraints = RelayConstraints(
            locations: .only(UserSelectedRelays(locations: [.country("INVALID COUNTRY")]))
        )

        let selectedRelay = try XCTUnwrap(RelaySelector.closestShadowsocksRelayConstrained(
            by: constraints,
            in: sampleRelays
        ))

        XCTAssertTrue(sampleRelays.bridge.relays.contains(selectedRelay))
    }

    func testRelayFilterConstraintWithOwnedOwnership() throws {
        let filter = RelayFilter(ownership: .owned, providers: .any)

        let constraints = RelayConstraints(
            locations: .only(UserSelectedRelays(locations: [.hostname("se", "sto", "se6-wireguard")])),
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
            locations: .only(UserSelectedRelays(locations: [.hostname("se", "sto", "se6-wireguard")])),
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
            locations: .only(UserSelectedRelays(locations: [.hostname("se", "sto", "se6-wireguard")])),
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
            locations: .only(UserSelectedRelays(locations: [.hostname("se", "sto", "se6-wireguard")])),
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
