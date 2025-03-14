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
    var obfuscation: ObfuscatorPortSelection!

    override func setUpWithError() throws {
        obfuscation = try ObfuscatorPortSelector(relays: sampleRelays)
            .obfuscate(tunnelSettings: LatestTunnelSettings(), connectionAttemptCount: 0)
    }

    // MARK: Single-/multihop

    func testSinglehopPicker() throws {
        let constraints = RelayConstraints(
            entryLocations: .only(UserSelectedRelays(locations: [.hostname("se", "sto", "se2-wireguard")])),
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")]))
        )

        let picker = SinglehopPicker(
            obfuscation: obfuscation,
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
            obfuscation: obfuscation,
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
            obfuscation: obfuscation,
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

    // MARK: DAITA/Direct only

    // DAITA - ON, Direct only - OFF, Multihop - OFF, Exit supports DAITA - FALSE
    // Direct only is off, so we should automatically pick the entry that is closest to exit.
    func testDirectOnlyOffDaitaOnForSinglehopWithoutDaitaRelay() throws {
        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se10-wireguard")]))
        )

        let picker = SinglehopPicker(
            obfuscation: obfuscation,
            constraints: constraints,
            connectionAttemptCount: 0,
            daitaSettings: DAITASettings(daitaState: .on, directOnlyState: .off)
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

        let picker = SinglehopPicker(
            obfuscation: obfuscation,
            constraints: constraints,
            connectionAttemptCount: 0,
            daitaSettings: DAITASettings(daitaState: .on, directOnlyState: .on)
        )

        XCTAssertThrowsError(try picker.pick())
    }

    // DAITA - ON, Direct only - OFF, Multihop - OFF, Exit supports DAITA - TRUE
    // Select the DAITA entry, no automatic routing needed.
    func testDirectOnlyOffDaitaOnForSinglehopWithDaitaRelay() throws {
        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("es", "mad", "es1-wireguard")]))
        )

        let picker = SinglehopPicker(
            obfuscation: obfuscation,
            constraints: constraints,
            connectionAttemptCount: 0,
            daitaSettings: DAITASettings(daitaState: .on, directOnlyState: .off)
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

        let picker = SinglehopPicker(
            obfuscation: obfuscation,
            constraints: constraints,
            connectionAttemptCount: 0,
            daitaSettings: DAITASettings(daitaState: .on, directOnlyState: .on)
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

        let picker = MultihopPicker(
            obfuscation: obfuscation,
            constraints: constraints,
            connectionAttemptCount: 0,
            daitaSettings: DAITASettings(daitaState: .on, directOnlyState: .off)
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

        let picker = MultihopPicker(
            obfuscation: obfuscation,
            constraints: constraints,
            connectionAttemptCount: 0,
            daitaSettings: DAITASettings(daitaState: .on, directOnlyState: .off)
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

        let picker = MultihopPicker(
            obfuscation: obfuscation,
            constraints: constraints,
            connectionAttemptCount: 0,
            daitaSettings: DAITASettings(daitaState: .on, directOnlyState: .on)
        )

        XCTAssertThrowsError(try picker.pick())
    }

    // MARK: Obfuscation

    func testObfuscationForSinglehop() throws {
        let constraints = RelayConstraints(entryLocations: .any, exitLocations: .any, port: .only(5000))
        let tunnelSettings = LatestTunnelSettings(
            wireGuardObfuscation: WireGuardObfuscationSettings(
                state: .udpOverTcp,
                udpOverTcpPort: .port80
            )
        )

        obfuscation = try ObfuscatorPortSelector(relays: sampleRelays)
            .obfuscate(tunnelSettings: tunnelSettings, connectionAttemptCount: 0)

        let picker = SinglehopPicker(
            obfuscation: obfuscation,
            constraints: constraints,
            connectionAttemptCount: 0,
            daitaSettings: DAITASettings()
        )

        let selectedRelays = try picker.pick()

        XCTAssertNil(selectedRelays.entry)
        XCTAssertEqual(selectedRelays.exit.endpoint.ipv4Relay.port, 80)
    }

    // If DAITA is on, the selected relay has DAITA and shadowsocks obfuscation yields no compatible relays,
    // we should make sure that .noObfuscatedRelaysFound is thrown rather than triggering smart routing.
    func testIncompatibleShadowsocksObfuscationNotTriggeringMultihop() throws {
        let constraints = RelayConstraints(
            entryLocations: .any,
            exitLocations: .only(UserSelectedRelays(locations: [.country("us")])),
            port: .only(5000)
        )
        let tunnelSettings = LatestTunnelSettings(
            wireGuardObfuscation: WireGuardObfuscationSettings(
                state: .shadowsocks,
                shadowsocksPort: .custom(1)
            )
        )

        obfuscation = try ObfuscatorPortSelector(relays: sampleRelays)
            .obfuscate(tunnelSettings: tunnelSettings, connectionAttemptCount: 0)

        let picker = SinglehopPicker(
            obfuscation: obfuscation,
            constraints: constraints,
            connectionAttemptCount: 0,
            daitaSettings: DAITASettings(daitaState: .on)
        )

        do {
            _ = try picker.pick()
            XCTFail("Should have thrown error due to incompatible shadowsocks obfuscation")
        } catch let error as NoRelaysSatisfyingConstraintsError {
            XCTAssertEqual(error.reason, .noObfuscatedRelaysFound)
        }
    }

    func testObfuscationForMultihop() throws {
        let constraints = RelayConstraints(entryLocations: .any, exitLocations: .any, port: .only(5000))
        let tunnelSettings = LatestTunnelSettings(
            wireGuardObfuscation: WireGuardObfuscationSettings(
                state: .udpOverTcp,
                udpOverTcpPort: .port80
            )
        )

        obfuscation = try ObfuscatorPortSelector(relays: sampleRelays)
            .obfuscate(tunnelSettings: tunnelSettings, connectionAttemptCount: 0)

        let picker = MultihopPicker(
            obfuscation: obfuscation,
            constraints: constraints,
            connectionAttemptCount: 0,
            daitaSettings: DAITASettings()
        )

        let selectedRelays = try picker.pick()

        XCTAssertEqual(selectedRelays.entry?.endpoint.ipv4Relay.port, 80)
        XCTAssertEqual(selectedRelays.exit.endpoint.ipv4Relay.port, 5000)
    }
}
