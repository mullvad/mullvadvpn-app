//
//  RelayPickingTests.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2024-06-14.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadMockData
import XCTest

@testable import MullvadREST
@testable import MullvadSettings
@testable import MullvadTypes

class RelayPickingTests: XCTestCase {
    let sampleRelays = ServerRelaysResponseStubs.sampleRelays
    var obfuscation: RelayObfuscation!

    override func setUpWithError() throws {
        // Default obfuscation settings to satisfy picker constructors for the tests below.
        obfuscation = RelayObfuscator(
            relays: sampleRelays,
            tunnelSettings: LatestTunnelSettings(),
            connectionAttemptCount: 0,
            obfuscationBypass: IdentityObfuscationProvider()
        ).obfuscate()
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

        // One of the five DAITA relays in Madrid is the closest.
        XCTAssertEqual(selectedRelays.entry?.location.cityCode, "mad")
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

        // One of the five DAITA relays in Madrid is the closest.
        XCTAssertEqual(selectedRelays.entry?.location.cityCode, "mad")
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

        // One of the five DAITA relays in Madrid is the closest.
        XCTAssertEqual(selectedRelays.entry?.location.cityCode, "mad")
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

    // DAITA - ON, Direct only - ON, Entry supports DAITA - TRUE, Entry does not support QUIC
    // Shadowsocks obfuscation should be picked instead of QUIC since entry does not support it
    func testMultihopCannotPickAutomaticallyInvalidObfuscation() throws {
        let constraints = RelayConstraints(
            entryLocations: .only(UserSelectedRelays(locations: [.hostname("us", "dal", "us-dal-wg-001")])),
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")]))
        )

        var settings = LatestTunnelSettings()
        settings.relayConstraints = constraints
        settings.daita = DAITASettings(daitaState: .on, directOnlyState: .on)

        // Mimic the obfuscator ran by the relay selector wrapper prior to invoking the `MultihopPicker`
        obfuscation = RelayObfuscator(
            relays: sampleRelays,
            tunnelSettings: LatestTunnelSettings(),
            connectionAttemptCount: 2,
            obfuscationBypass: IdentityObfuscationProvider()
        ).obfuscate()

        // It will already have pre-filtered relays to select obfuscation via QUIC because it's the 2nd connection attempt
        XCTAssertEqual(obfuscation.method, .quic)

        // The `MultihopPicker` will re-roll an obfuscator to find out that QUIC is not supported for the selected entry
        // It will then fallback to picking shadowsocks obfuscation instead
        let picker = MultihopPicker(
            obfuscation: obfuscation,
            tunnelSettings: settings,
            connectionAttemptCount: 2
        )

        let selectedRelays = try picker.pick()

        XCTAssertEqual(selectedRelays.obfuscation, .udpOverTcp)
        XCTAssertEqual(selectedRelays.entry?.hostname, "us-dal-wg-001")
        XCTAssertEqual(selectedRelays.exit.hostname, "se10-wireguard")
    }

    // DAITA - OFF, Entry does not support QUIC
    // Shadowsocks obfuscation should be picked instead of QUIC since entry does not support it
    func testSinglehopCannotPickAutomaticallyInvalidObfuscation() throws {
        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("us", "dal", "us-dal-wg-001")]))
        )

        var settings = LatestTunnelSettings()
        settings.relayConstraints = constraints

        // Mimic the obfuscator ran by the relay selector wrapper prior to invoking the `SinglehopPicker`
        obfuscation = RelayObfuscator(
            relays: sampleRelays,
            tunnelSettings: LatestTunnelSettings(),
            connectionAttemptCount: 2,
            obfuscationBypass: IdentityObfuscationProvider()
        ).obfuscate()

        // It will already have pre-filtered relays to select obfuscation via QUIC because it's the 2nd connection attempt
        XCTAssertEqual(obfuscation.method, .quic)

        let picker = SinglehopPicker(
            obfuscation: obfuscation,
            tunnelSettings: settings,
            connectionAttemptCount: 2
        )

        let selectedRelays = try picker.pick()
        XCTAssertEqual(selectedRelays.obfuscation, .udpOverTcp)
        XCTAssertEqual(selectedRelays.entry?.hostname, nil)
        XCTAssertEqual(selectedRelays.exit.hostname, "us-dal-wg-001")
    }
}
