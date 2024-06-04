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

    // MARK: - single-Hop tests

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

        let constrainedLocations = RelaySelector.applyConstraints(
            constraints.exitLocations,
            filterConstraint: constraints.filter,
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
        XCTAssertTrue(allPorts.contains(result.endpoint.ipv4Relay.port))

        result = try pickRelay(by: constraints, in: sampleRelays, failedAttemptCount: 2)
        XCTAssertEqual(result.endpoint.ipv4Relay.port, defaultPort)

        result = try pickRelay(by: constraints, in: sampleRelays, failedAttemptCount: 3)
        XCTAssertEqual(result.endpoint.ipv4Relay.port, defaultPort)

        result = try pickRelay(by: constraints, in: sampleRelays, failedAttemptCount: 4)
        XCTAssertTrue(allPorts.contains(result.endpoint.ipv4Relay.port))
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
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "sto", "se6-wireguard")])),
            filter: .only(filter)
        )

        XCTAssertThrowsError(try pickRelay(by: constraints, in: sampleRelays, failedAttemptCount: 0))
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
        let provider = "DataPacket"
        let filter = RelayFilter(ownership: .any, providers: .only([provider]))

        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "sto", "se6-wireguard")])),
            filter: .only(filter)
        )

        XCTAssertThrowsError(try pickRelay(by: constraints, in: sampleRelays, failedAttemptCount: 0))
    }
}

extension RelaySelectorTests {
    private func pickRelay(
        by constraints: RelayConstraints,
        in relays: REST.ServerRelaysResponse,
        failedAttemptCount: UInt
    ) throws -> RelaySelectorMatch {
        let candidates = try RelaySelector.WireGuard.findCandidates(
            by: constraints.exitLocations,
            in: relays,
            filterConstraint: constraints.filter
        )

        return try RelaySelector.WireGuard.pickCandidate(
            from: candidates,
            relays: relays,
            portConstraint: constraints.port,
            numberOfFailedAttempts: failedAttemptCount
        )
    }
}
