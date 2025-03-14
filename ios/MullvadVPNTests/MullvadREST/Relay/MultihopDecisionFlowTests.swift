//
//  MultihopDecisionFlowTests.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2024-06-14.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadMockData
@testable import MullvadREST
@testable import MullvadSettings
@testable import MullvadTypes
import XCTest

class MultihopDecisionFlowTests: XCTestCase {
    let sampleRelays = ServerRelaysResponseStubs.sampleRelays

    func testOneToOneCanHandle() throws {
        let oneToOne = OneToOne(next: nil, relayPicker: picker)

        XCTAssertTrue(oneToOne.canHandle(
            entryCandidates: [seSto6],
            exitCandidates: [seSto2]
        ))

        XCTAssertFalse(oneToOne.canHandle(
            entryCandidates: [seSto2, seSto6],
            exitCandidates: [seSto2]
        ))

        XCTAssertFalse(oneToOne.canHandle(
            entryCandidates: [seSto2, seSto6],
            exitCandidates: [seSto2, seSto6]
        ))
    }

    func testOneToManyCanHandle() throws {
        let oneToMany = OneToMany(next: nil, relayPicker: picker)

        XCTAssertTrue(oneToMany.canHandle(
            entryCandidates: [seSto2],
            exitCandidates: [seSto2, seSto6]
        ))

        XCTAssertFalse(oneToMany.canHandle(
            entryCandidates: [seSto6],
            exitCandidates: [seSto2]
        ))

        XCTAssertFalse(oneToMany.canHandle(
            entryCandidates: [seSto2, seSto6],
            exitCandidates: [seSto2, seSto6]
        ))
    }

    func testManyToOneCanHandle() throws {
        let manyToOne = ManyToOne(next: nil, relayPicker: picker)

        XCTAssertTrue(manyToOne.canHandle(
            entryCandidates: [seSto2, seSto6],
            exitCandidates: [seSto2]
        ))

        XCTAssertFalse(manyToOne.canHandle(
            entryCandidates: [seSto6],
            exitCandidates: [seSto2]
        ))

        XCTAssertFalse(manyToOne.canHandle(
            entryCandidates: [seSto2, seSto6],
            exitCandidates: [seSto2, seSto6]
        ))
    }

    func testManyToManyCanHandle() throws {
        let manyToMany = ManyToMany(next: nil, relayPicker: picker)

        XCTAssertTrue(manyToMany.canHandle(
            entryCandidates: [seSto2, seSto6],
            exitCandidates: [seSto2, seSto6]
        ))

        XCTAssertFalse(manyToMany.canHandle(
            entryCandidates: [seSto6],
            exitCandidates: [seSto2]
        ))

        XCTAssertFalse(manyToMany.canHandle(
            entryCandidates: [seSto2, seSto6],
            exitCandidates: [seSto2]
        ))
    }

    func testOneToOnePick() throws {
        let oneToOne = OneToOne(next: nil, relayPicker: picker)

        let entryCandidates = [seSto2]
        let exitCandidates = [seSto6]

        let selectedRelays = try oneToOne.pick(
            entryCandidates: entryCandidates,
            exitCandidates: exitCandidates,
            daitaAutomaticRouting: false
        )

        XCTAssertEqual(selectedRelays.entry?.hostname, "se2-wireguard")
        XCTAssertEqual(selectedRelays.exit.hostname, "se6-wireguard")
    }

    func testOneToManyPick() throws {
        let oneToMany = OneToMany(next: nil, relayPicker: picker)

        let entryCandidates = [seSto2]
        let exitCandidates = [seSto2, seSto6]

        let selectedRelays = try oneToMany.pick(
            entryCandidates: entryCandidates,
            exitCandidates: exitCandidates,
            daitaAutomaticRouting: false
        )

        XCTAssertEqual(selectedRelays.entry?.hostname, "se2-wireguard")
        XCTAssertEqual(selectedRelays.exit.hostname, "se6-wireguard")
    }

    func testManyToOnePick() throws {
        let manyToOne = ManyToOne(next: nil, relayPicker: picker)

        let entryCandidates = [seSto2, seSto6]
        let exitCandidates = [seSto2]

        let selectedRelays = try manyToOne.pick(
            entryCandidates: entryCandidates,
            exitCandidates: exitCandidates,
            daitaAutomaticRouting: false
        )

        XCTAssertEqual(selectedRelays.entry?.hostname, "se6-wireguard")
        XCTAssertEqual(selectedRelays.exit.hostname, "se2-wireguard")
    }

    func testManyToManyPick() throws {
        let manyToMany = ManyToMany(next: nil, relayPicker: picker)

        let entryCandidates = [seSto2, seSto6]
        let exitCandidates = [seSto2, seSto6]

        let selectedRelays = try manyToMany.pick(
            entryCandidates: entryCandidates,
            exitCandidates: exitCandidates,
            daitaAutomaticRouting: false
        )

        if selectedRelays.exit.hostname == "se2-wireguard" {
            XCTAssertEqual(selectedRelays.entry?.hostname, "se6-wireguard")
        } else {
            XCTAssertEqual(selectedRelays.entry?.hostname, "se2-wireguard")
        }
    }
}

extension MultihopDecisionFlowTests {
    var picker: MultihopPicker {
        let obfuscation = try? ObfuscatorPortSelector(relays: sampleRelays)
            .obfuscate(tunnelSettings: LatestTunnelSettings(), connectionAttemptCount: 0)

        let constraints = RelayConstraints(
            entryLocations: .only(UserSelectedRelays(locations: [.city("se", "sto")])),
            exitLocations: .only(UserSelectedRelays(locations: [.city("se", "sto")]))
        )

        return MultihopPicker(
            obfuscation: obfuscation.unsafelyUnwrapped,
            constraints: constraints,
            connectionAttemptCount: 0,
            daitaSettings: DAITASettings(daitaState: .off)
        )
    }

    var seSto2: RelayWithLocation<REST.ServerRelay> {
        let relay = sampleRelays.wireguard.relays.first { $0.hostname == "se2-wireguard" }!
        let serverLocation = sampleRelays.locations["se-sto"]!
        let location = Location(
            country: serverLocation.country,
            countryCode: serverLocation.country,
            city: serverLocation.city,
            cityCode: "se-sto",
            latitude: serverLocation.latitude,
            longitude: serverLocation.longitude
        )

        return RelayWithLocation(relay: relay, serverLocation: location)
    }

    var seSto6: RelayWithLocation<REST.ServerRelay> {
        let relay = sampleRelays.wireguard.relays.first { $0.hostname == "se6-wireguard" }!
        let serverLocation = sampleRelays.locations["se-sto"]!
        let location = Location(
            country: serverLocation.country,
            countryCode: serverLocation.country,
            city: serverLocation.city,
            cityCode: "se-sto",
            latitude: serverLocation.latitude,
            longitude: serverLocation.longitude
        )

        return RelayWithLocation(relay: relay, serverLocation: location)
    }
}
