//
//  RelayPickingTests.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2024-06-14.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadMockData
import XCTest

@testable import MullvadREST
@testable import MullvadSettings
@testable import MullvadTypes

class RelayPickingTests: XCTestCase {
    let sampleRelays = ServerRelaysResponseStubs.sampleRelays

    // MARK: Single-/multihop

    func testSinglehopPicker() throws {
        let constraints = RelayConstraints(
            entryLocations: .only(UserSelectedRelays(locations: [.hostname("se", "sto", "se2-wireguard")])),
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")]))
        )

        var settings = LatestTunnelSettings()
        settings.relayConstraints = constraints

        let picker = SinglehopPicker(
            relays: sampleRelays,
            tunnelSettings: settings,
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

        var settings = LatestTunnelSettings()
        settings.relayConstraints = constraints

        let picker = MultihopPicker(
            relays: sampleRelays,
            tunnelSettings: settings,
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

        var settings = LatestTunnelSettings()
        settings.relayConstraints = constraints

        let picker = MultihopPicker(
            relays: sampleRelays,
            tunnelSettings: settings,
            connectionAttemptCount: 0
        )

        XCTAssertThrowsError(
            try picker.pick()
        ) { error in
            let error = error as? NoRelaysSatisfyingConstraintsError
            XCTAssertEqual(error?.reason, .entryEqualsExit)
        }
    }

    // MARK: DAITA/Automatic routing

    // DAITA - ON, Multihop - WHENNEEDED, Exit supports DAITA - FALSE
    // .whenNeeded is on, so we should automatically pick the entry that is closest to exit.
    func testMultihopWhenNeededDaitaOnForSinglehopWithoutDaitaRelay() throws {
        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")]))
        )

        var settings = LatestTunnelSettings()
        settings.relayConstraints = constraints
        settings.daita = DAITASettings(daitaState: .on)
        settings.tunnelMultihopState = .whenNeeded

        let picker = SinglehopPicker(
            relays: sampleRelays,
            tunnelSettings: settings,
            connectionAttemptCount: 0
        )

        let selectedRelays = try picker.pick()

        // One of the five DAITA relays in Madrid is the closest.
        XCTAssertEqual(selectedRelays.entry?.location.cityCode, "mad")
        XCTAssertEqual(selectedRelays.exit.hostname, "se10-wireguard")
    }

    // DAITA - ON, Multihop - NEVER, Exit supports DAITA - FALSE
    // Go into blocked state since single hop requires a DAITA entry.
    func testMultihopNeverDaitaOnForSinglehopWithoutDaitaRelay() throws {
        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")]))
        )

        var settings = LatestTunnelSettings()
        settings.relayConstraints = constraints
        settings.daita = DAITASettings(daitaState: .on)

        let picker = SinglehopPicker(
            relays: sampleRelays,
            tunnelSettings: settings,
            connectionAttemptCount: 0
        )

        XCTAssertThrowsError(try picker.pick())
    }

    // DAITA - ON, Multihop - WHENNEEDED, Exit supports DAITA - TRUE
    // Select the DAITA entry, no automatic routing needed.
    func testMultihopWhenNeededDaitaOnForSinglehopWithDaitaRelay() throws {
        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("es", "mad", "es1-wireguard")]))
        )

        var settings = LatestTunnelSettings()
        settings.relayConstraints = constraints
        settings.daita = DAITASettings(daitaState: .on)
        settings.tunnelMultihopState = .whenNeeded

        let picker = SinglehopPicker(
            relays: sampleRelays,
            tunnelSettings: settings,
            connectionAttemptCount: 0
        )

        let selectedRelays = try picker.pick()

        XCTAssertNil(selectedRelays.entry)
        XCTAssertEqual(selectedRelays.exit.hostname, "es1-wireguard")
    }

    // DAITA - ON, Multihop - NEVER, Exit supports DAITA - TRUE
    // Select the DAITA entry.
    func testMultihopNeverDaitaOnForSinglehopWithDaitaRelay() throws {
        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("es", "mad", "es1-wireguard")]))
        )

        var settings = LatestTunnelSettings()
        settings.relayConstraints = constraints
        settings.daita = DAITASettings(daitaState: .on)

        let picker = SinglehopPicker(
            relays: sampleRelays,
            tunnelSettings: settings,
            connectionAttemptCount: 0
        )

        let selectedRelays = try picker.pick()

        XCTAssertNil(selectedRelays.entry?.hostname)
        XCTAssertEqual(selectedRelays.exit.hostname, "es1-wireguard")
    }

    // DAITA - ON, Multihop - WHENNEEDED, Entry supports DAITA - TRUE
    // .whenNeeded is on, so we should automatically pick the entry that is closest to exit, ignoring
    // selected multihop entry.
    func testMultihopWhenNeededDaitaOnForMultihopWithDaitaRelay() throws {
        let constraints = RelayConstraints(
            entryLocations: .only(UserSelectedRelays(locations: [.hostname("us", "nyc", "us-nyc-wg-301")])),
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")]))
        )

        var settings = LatestTunnelSettings()
        settings.relayConstraints = constraints
        settings.daita = DAITASettings(daitaState: .on)
        settings.tunnelMultihopState = .whenNeeded

        let picker = MultihopPicker(
            relays: sampleRelays,
            tunnelSettings: settings,
            connectionAttemptCount: 0
        )

        let selectedRelays = try picker.pick()

        // One of the five DAITA relays in Madrid is the closest.
        XCTAssertEqual(selectedRelays.entry?.location.cityCode, "mad")
        XCTAssertEqual(selectedRelays.exit.hostname, "se10-wireguard")
    }

    // DAITA - ON, Multihop - WHENNEEDED, Entry supports DAITA - FALSE
    // .whenNeeded is on, so we should automatically pick the entry that is closest to exit, ignoring
    // selected multihop entry.
    func testMultihopWhenNeededDaitaOnForMultihopWithoutDaitaRelay() throws {
        let constraints = RelayConstraints(
            entryLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")])),
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")]))
        )

        var settings = LatestTunnelSettings()
        settings.relayConstraints = constraints
        settings.daita = DAITASettings(daitaState: .on)
        settings.tunnelMultihopState = .whenNeeded

        let picker = MultihopPicker(
            relays: sampleRelays,
            tunnelSettings: settings,
            connectionAttemptCount: 0
        )

        let selectedRelays = try picker.pick()

        // One of the five DAITA relays in Madrid is the closest.
        XCTAssertEqual(selectedRelays.entry?.location.cityCode, "mad")
        XCTAssertEqual(selectedRelays.exit.hostname, "se10-wireguard")
    }

    // DAITA - ON, Multihop - ALWAYS, Entry supports DAITA - FALSE
    // Go into blocked state since .always requires a DAITA entry.
    func testMultihopAlwaysDaitaOnForMultihopWithoutDaitaRelay() throws {
        let constraints = RelayConstraints(
            entryLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")])),
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")]))
        )

        var settings = LatestTunnelSettings()
        settings.relayConstraints = constraints
        settings.daita = DAITASettings(daitaState: .on)
        settings.tunnelMultihopState = .always

        let picker = MultihopPicker(
            relays: sampleRelays,
            tunnelSettings: settings,
            connectionAttemptCount: 0
        )

        XCTAssertThrowsError(try picker.pick())
    }

    // QUIC - ON, Entry supports QUIC - TRUE, Exit supports QUIC - FALSE
    // Entry supports QUIC and thus should yield no errors.
    func testQuicOnForMultihopWithQuicRelay() throws {
        let constraints = RelayConstraints(
            entryLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")])),
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "sto", "se2-wireguard")]))
        )

        var settings = LatestTunnelSettings()
        settings.relayConstraints = constraints
        settings.wireGuardObfuscation = .init(state: .quic)

        let picker = MultihopPicker(
            relays: sampleRelays,
            tunnelSettings: settings,
            connectionAttemptCount: 0
        )

        XCTAssertNoThrow(try picker.pick())
    }
}
