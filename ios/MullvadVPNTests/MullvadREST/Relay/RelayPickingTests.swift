//
//  RelayPickingTests.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2024-06-14.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

import MullvadMockData
@testable import MullvadREST
@testable import MullvadSettings
@testable import MullvadTypes
import XCTest

class RelayPickingTests: XCTestCase {
    let sampleRelays = ServerRelaysResponseStubs.sampleRelays
    var obfuscation: RelayObfuscation!

    override func setUpWithError() throws {
        obfuscation = try RelayObfuscator(relays: sampleRelays)
            .obfuscate(tunnelSettings: LatestTunnelSettings(), connectionAttemptCount: 0)
    }

    // MARK: Single-/multihop

    func testSinglehopPicker() throws {
        let constraints = RelayConstraints(
            entryLocations: .only(UserSelectedRelays(locations: [.hostname("se", "sto", "se2-wireguard")])),
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")]))
        )

        var settings = LatestTunnelSettings()
        settings.relayConstraints = constraints

        let picker = SinglehopPicker(
            obfuscation: obfuscation,
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
            obfuscation: obfuscation,
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
            obfuscation: obfuscation,
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

    // MARK: DAITA/Direct only

    // DAITA - ON, Direct only - OFF, Multihop - OFF, Exit supports DAITA - FALSE
    // Direct only is off, so we should automatically pick the entry that is closest to exit.
    func testDirectOnlyOffDaitaOnForSinglehopWithoutDaitaRelay() throws {
        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")]))
        )

        var settings = LatestTunnelSettings()
        settings.relayConstraints = constraints
        settings.daita = DAITASettings(daitaState: .on)

        let picker = SinglehopPicker(
            obfuscation: obfuscation,
            tunnelSettings: settings,
            connectionAttemptCount: 0
        )

        let selectedRelays = try picker.pick()

        XCTAssertEqual(selectedRelays.entry?.hostname, "es1-wireguard") // Madrid relay is closest to exit relay.
        XCTAssertEqual(selectedRelays.exit.hostname, "se10-wireguard")
    }

    // DAITA - ON, Direct only - ON, Multihop - OFF, Exit supports DAITA - FALSE
    // Go into blocked state since Direct only requires a DAITA entry.
    func testDirectOnlyOnDaitaOnForSinglehopWithoutDaitaRelay() throws {
        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")]))
        )

        var settings = LatestTunnelSettings()
        settings.relayConstraints = constraints
        settings.daita = DAITASettings(daitaState: .on, directOnlyState: .on)

        let picker = SinglehopPicker(
            obfuscation: obfuscation,
            tunnelSettings: settings,
            connectionAttemptCount: 0
        )

        XCTAssertThrowsError(try picker.pick())
    }

    // DAITA - ON, Direct only - OFF, Multihop - OFF, Exit supports DAITA - TRUE
    // Select the DAITA entry, no automatic routing needed.
    func testDirectOnlyOffDaitaOnForSinglehopWithDaitaRelay() throws {
        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("es", "mad", "es1-wireguard")]))
        )

        var settings = LatestTunnelSettings()
        settings.relayConstraints = constraints
        settings.daita = DAITASettings(daitaState: .on)

        let picker = SinglehopPicker(
            obfuscation: obfuscation,
            tunnelSettings: settings,
            connectionAttemptCount: 0
        )

        let selectedRelays = try picker.pick()

        XCTAssertNil(selectedRelays.entry)
        XCTAssertEqual(selectedRelays.exit.hostname, "es1-wireguard")
    }

    // DAITA - ON, Direct only - ON, Multihop - OFF, Exit supports DAITA - TRUE
    // Select the DAITA entry.
    func testDirectOnlyOnDaitaOnForSinglehopWithDaitaRelay() throws {
        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("es", "mad", "es1-wireguard")]))
        )

        var settings = LatestTunnelSettings()
        settings.relayConstraints = constraints
        settings.daita = DAITASettings(daitaState: .on, directOnlyState: .on)

        let picker = SinglehopPicker(
            obfuscation: obfuscation,
            tunnelSettings: settings,
            connectionAttemptCount: 0
        )

        let selectedRelays = try picker.pick()

        XCTAssertNil(selectedRelays.entry?.hostname)
        XCTAssertEqual(selectedRelays.exit.hostname, "es1-wireguard")
    }

    // DAITA - ON, Direct only - OFF, Multihop - ON, Entry supports DAITA - TRUE
    // Direct only is off, so we should automatically pick the entry that is closest to exit, ignoring
    // selected multihop entry.
    func testDirectOnlyOffDaitaOnForMultihopWithDaitaRelay() throws {
        let constraints = RelayConstraints(
            entryLocations: .only(UserSelectedRelays(locations: [.hostname("us", "nyc", "us-nyc-wg-301")])),
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")]))
        )

        var settings = LatestTunnelSettings()
        settings.relayConstraints = constraints
        settings.daita = DAITASettings(daitaState: .on)

        let picker = MultihopPicker(
            obfuscation: obfuscation,
            tunnelSettings: settings,
            connectionAttemptCount: 0
        )

        let selectedRelays = try picker.pick()

        XCTAssertEqual(selectedRelays.entry?.hostname, "es1-wireguard") // Madrid relay is closest to exit relay.
        XCTAssertEqual(selectedRelays.exit.hostname, "se10-wireguard")
    }

    // DAITA - ON, Direct only - OFF, Multihop - ON, Entry supports DAITA - FALSE
    // Direct only is off, so we should automatically pick the entry that is closest to exit, ignoring
    // selected multihop entry.
    func testDirectOnlyOffDaitaOnForMultihopWithoutDaitaRelay() throws {
        let constraints = RelayConstraints(
            entryLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")])),
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")]))
        )

        var settings = LatestTunnelSettings()
        settings.relayConstraints = constraints
        settings.daita = DAITASettings(daitaState: .on)

        let picker = MultihopPicker(
            obfuscation: obfuscation,
            tunnelSettings: settings,
            connectionAttemptCount: 0
        )

        let selectedRelays = try picker.pick()

        XCTAssertEqual(selectedRelays.entry?.hostname, "es1-wireguard") // Madrid relay is closest to exit relay.
        XCTAssertEqual(selectedRelays.exit.hostname, "se10-wireguard")
    }

    // DAITA - ON, Direct only - ON, Multihop - ON, Entry supports DAITA - FALSE
    // Go into blocked state since Direct only requires a DAITA entry.
    func testDirectOnlyOnDaitaOnForMultihopWithoutDaitaRelay() throws {
        let constraints = RelayConstraints(
            entryLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")])),
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")]))
        )

        var settings = LatestTunnelSettings()
        settings.relayConstraints = constraints
        settings.daita = DAITASettings(daitaState: .on, directOnlyState: .on)

        let picker = MultihopPicker(
            obfuscation: obfuscation,
            tunnelSettings: settings,
            connectionAttemptCount: 0
        )

        XCTAssertThrowsError(try picker.pick())
    }

    // MARK: Obfuscation

    func testObfuscationForSinglehop() throws {
        let constraints = RelayConstraints(entryLocations: .any, exitLocations: .any, port: .only(5000))
        let settings = LatestTunnelSettings(
            relayConstraints: constraints,
            wireGuardObfuscation: WireGuardObfuscationSettings(
                state: .udpOverTcp,
                udpOverTcpPort: .port80
            )
        )

        obfuscation = try RelayObfuscator(relays: sampleRelays)
            .obfuscate(tunnelSettings: settings, connectionAttemptCount: 0)

        let picker = SinglehopPicker(
            obfuscation: obfuscation,
            tunnelSettings: settings,
            connectionAttemptCount: 0
        )

        let selectedRelays = try picker.pick()

        XCTAssertNil(selectedRelays.entry)
        XCTAssertEqual(selectedRelays.exit.endpoint.ipv4Relay.port, 80)
    }

    func testObfuscationForMultihop() throws {
        let constraints = RelayConstraints(entryLocations: .any, exitLocations: .any, port: .only(5000))
        let settings = LatestTunnelSettings(
            relayConstraints: constraints,
            wireGuardObfuscation: WireGuardObfuscationSettings(
                state: .udpOverTcp,
                udpOverTcpPort: .port80
            )
        )

        obfuscation = try RelayObfuscator(relays: sampleRelays)
            .obfuscate(tunnelSettings: settings, connectionAttemptCount: 0)

        let picker = MultihopPicker(
            obfuscation: obfuscation,
            tunnelSettings: settings,
            connectionAttemptCount: 0
        )

        let selectedRelays = try picker.pick()

        XCTAssertEqual(selectedRelays.entry?.endpoint.ipv4Relay.port, 80)
        XCTAssertEqual(selectedRelays.exit.endpoint.ipv4Relay.port, 5000)
    }
}
