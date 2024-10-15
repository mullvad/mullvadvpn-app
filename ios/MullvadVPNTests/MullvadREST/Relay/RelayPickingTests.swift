//
//  RelayPickingTests.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2024-06-14.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation

@testable import MullvadREST
@testable import MullvadSettings
@testable import MullvadTypes
import XCTest

class RelayPickingTests: XCTestCase {
    let sampleRelays = ServerRelaysResponseStubs.sampleRelays

    func testSinglehopPicker() throws {
        let constraints = RelayConstraints(
            entryLocations: .only(UserSelectedRelays(locations: [.hostname("se", "sto", "se2-wireguard")])),
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")]))
        )

        let picker = SinglehopPicker(
            relays: sampleRelays,
            constraints: constraints,
            connectionAttemptCount: 0,
            daitaSettings: DAITASettings()
        )

        let selectedRelays = try picker.pick()

        XCTAssertNil(selectedRelays.entry)
        XCTAssertEqual(selectedRelays.exit.hostname, "se10-wireguard")
    }

    func testMultihopPicker() throws {
        let constraints = RelayConstraints(
            entryLocations: .only(UserSelectedRelays(locations: [.hostname("se", "sto", "se2-wireguard")])),
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")]))
        )

        let picker = MultihopPicker(
            relays: sampleRelays,
            constraints: constraints,
            connectionAttemptCount: 0,
            daitaSettings: DAITASettings()
        )

        let selectedRelays = try picker.pick()

        XCTAssertEqual(selectedRelays.entry?.hostname, "se2-wireguard")
        XCTAssertEqual(selectedRelays.exit.hostname, "se10-wireguard")
    }

    func testMultihopPickerWithSameEntryAndExit() throws {
        let constraints = RelayConstraints(
            entryLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")])),
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")]))
        )

        let picker = MultihopPicker(
            relays: sampleRelays,
            constraints: constraints,
            connectionAttemptCount: 0,
            daitaSettings: DAITASettings()
        )

        XCTAssertThrowsError(
            try picker.pick()
        ) { error in
            let error = error as? NoRelaysSatisfyingConstraintsError
            XCTAssertEqual(error?.reason, .entryEqualsExit)
        }
    }

    func testDirectOnlyOffDaitaOnForSinglehopWithoutDaitaRelay() throws {
        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")]))
        )

        let picker = SinglehopPicker(
            relays: sampleRelays,
            constraints: constraints,
            connectionAttemptCount: 0,
            daitaSettings: DAITASettings(daitaState: .on, directOnlyState: .off)
        )

        let selectedRelays = try picker.pick()

        XCTAssertEqual(selectedRelays.entry?.hostname, "es1-wireguard") // Madrid relay is closest to exit relay.
        XCTAssertEqual(selectedRelays.exit.hostname, "se10-wireguard")
    }

    func testDirectOnlyOnDaitaOnForSinglehopWithoutDaitaRelay() throws {
        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")]))
        )

        let picker = SinglehopPicker(
            relays: sampleRelays,
            constraints: constraints,
            connectionAttemptCount: 0,
            daitaSettings: DAITASettings(daitaState: .on, directOnlyState: .on)
        )

        XCTAssertThrowsError(try picker.pick())
    }

    func testDirectOnlyOffDaitaOnForSinglehopWithDaitaRelay() throws {
        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("es", "mad", "es1-wireguard")]))
        )

        let picker = SinglehopPicker(
            relays: sampleRelays,
            constraints: constraints,
            connectionAttemptCount: 0,
            daitaSettings: DAITASettings(daitaState: .on, directOnlyState: .off)
        )

        let selectedRelays = try picker.pick()

        XCTAssertNil(selectedRelays.entry?.hostname)
        XCTAssertEqual(selectedRelays.exit.hostname, "es1-wireguard")
    }

    func testDirectOnlyOnDaitaOnForSinglehopWithDaitaRelay() throws {
        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("es", "mad", "es1-wireguard")]))
        )

        let picker = SinglehopPicker(
            relays: sampleRelays,
            constraints: constraints,
            connectionAttemptCount: 0,
            daitaSettings: DAITASettings(daitaState: .on, directOnlyState: .on)
        )

        let selectedRelays = try picker.pick()

        XCTAssertNil(selectedRelays.entry?.hostname)
        XCTAssertEqual(selectedRelays.exit.hostname, "es1-wireguard")
    }

    func testDirectOnlyOffDaitaOnForMultihopWithDaitaRelay() throws {
        let constraints = RelayConstraints(
            entryLocations: .only(UserSelectedRelays(locations: [.hostname("us", "nyc", "us-nyc-wg-301")])),
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")]))
        )

        let picker = SinglehopPicker(
            relays: sampleRelays,
            constraints: constraints,
            connectionAttemptCount: 0,
            daitaSettings: DAITASettings(daitaState: .on, directOnlyState: .off)
        )

        let selectedRelays = try picker.pick()

        XCTAssertEqual(selectedRelays.entry?.hostname, "es1-wireguard") // Madrid relay is closest to exit relay.
        XCTAssertEqual(selectedRelays.exit.hostname, "se10-wireguard")
    }

    func testDirectOnlyOffDaitaOnForMultihopWithoutDaitaRelay() throws {
        let constraints = RelayConstraints(
            entryLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")])),
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")]))
        )

        let picker = SinglehopPicker(
            relays: sampleRelays,
            constraints: constraints,
            connectionAttemptCount: 0,
            daitaSettings: DAITASettings(daitaState: .on, directOnlyState: .off)
        )

        let selectedRelays = try picker.pick()

        XCTAssertEqual(selectedRelays.entry?.hostname, "es1-wireguard") // Madrid relay is closest to exit relay.
        XCTAssertEqual(selectedRelays.exit.hostname, "se10-wireguard")
    }

    func testDirectOnlyOnDaitaOnForMultihopWithoutDaitaRelay() throws {
        let constraints = RelayConstraints(
            entryLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")])),
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")]))
        )

        let picker = SinglehopPicker(
            relays: sampleRelays,
            constraints: constraints,
            connectionAttemptCount: 0,
            daitaSettings: DAITASettings(daitaState: .on, directOnlyState: .on)
        )

        XCTAssertThrowsError(try picker.pick())
    }
}
