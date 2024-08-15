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
            constraints: constraints,
            daitaSettings: DAITASettings(state: .off),
            relays: sampleRelays,
            connectionAttemptCount: 0
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
            constraints: constraints,
            daitaSettings: DAITASettings(state: .off),
            relays: sampleRelays,
            connectionAttemptCount: 0
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
            constraints: constraints,
            daitaSettings: DAITASettings(state: .off),
            relays: sampleRelays,
            connectionAttemptCount: 0
        )

        XCTAssertThrowsError(
            try picker.pick()
        ) { error in
            let error = error as? NoRelaysSatisfyingConstraintsError
            XCTAssertEqual(error?.reason, .multihopEntryEqualsExit)
        }
    }
}
