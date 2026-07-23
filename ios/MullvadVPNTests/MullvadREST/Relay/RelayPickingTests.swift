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

        var settings = LatestTunnelSettings(tunnelMultihopState: .always)
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

        var settings = LatestTunnelSettings(tunnelMultihopState: .always)
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

        var settings = LatestTunnelSettings(tunnelMultihopState: .never)
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

    // QUIC enabled, multihop enabled when needed.
    // The selected exit relay is incompatible with the effective IP version,
    // so the picker should require multihop rather than use it as a single-hop exit.
    func testUsesMultihopWhenQuicExitRelayIsIncompatibleWithIpVersion() throws {
        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se3-wireguard")]))
        )

        var settings = LatestTunnelSettings()
        settings.relayConstraints = constraints
        settings.wireGuardObfuscation = .init(state: .quic)
        settings.ipVersion = .automatic
        settings.tunnelMultihopState = .whenNeeded

        let picker = SinglehopPicker(
            relays: sampleRelays,
            tunnelSettings: settings,
            connectionAttemptCount: 0
        )

        let selectedRelays = try picker.pick()

        XCTAssertNotNil(selectedRelays.entry)
        XCTAssertEqual(selectedRelays.exit.hostname, "se3-wireguard")
    }

    // QUIC enabled, multihop enabled when needed.
    // The selected exit relay supports QUIC for the requested IPv6 connection,
    // so single-hop selection should succeed.
    func testDoesNotUseMultihopWhenQuicExitRelaySupportsIpv6() throws {
        let constraints = RelayConstraints(
            entryLocations: .any,
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se3-wireguard")]))
        )

        var settings = LatestTunnelSettings()
        settings.relayConstraints = constraints
        settings.wireGuardObfuscation = .init(state: .quic)
        settings.ipVersion = .ipv6
        settings.tunnelMultihopState = .whenNeeded

        let picker = SinglehopPicker(
            relays: sampleRelays,
            tunnelSettings: settings,
            connectionAttemptCount: 0
        )
        XCTAssertNoThrow(try picker.pick())
    }

    // Shadowsocks enabled with a custom port outside the default ranges.
    // The selected exit relay is incompatible with the effective IP version,
    // so the picker should select a compatible entry relay and use multihop.
    func testUsesMultihopWhenShadowsocksExitRelayIsIncompatibleWithIpVersion() throws {
        let allPorts: Range<UInt16> = 1..<65000
        let defaultPortRanges = RelaySelector.parseRawPortRanges(sampleRelays.wireguard.shadowsocksPortRanges)

        let portsOutsideDefaultRange = allPorts.filter { port in
            !defaultPortRanges.contains { range in
                range.contains(port)
            }
        }
        let port = try XCTUnwrap(portsOutsideDefaultRange.randomElement())

        let constraints = RelayConstraints(
            entryLocations: .any,
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se3-wireguard")]))
        )

        var settings = LatestTunnelSettings()
        settings.relayConstraints = constraints
        settings.wireGuardObfuscation = .init(state: .shadowsocks, shadowsocksPort: .custom(port))
        settings.ipVersion = .automatic
        settings.tunnelMultihopState = .whenNeeded

        let picker = MultihopPicker(
            relays: sampleRelays,
            tunnelSettings: settings,
            connectionAttemptCount: 0
        )

        let selectedRelays = try picker.pick()

        XCTAssertEqual(selectedRelays.entry?.location.countryCode, "se")
        XCTAssertEqual(selectedRelays.exit.hostname, "se3-wireguard")
    }

    // Shadowsocks enabled with a custom port outside the default ranges.
    // The selected exit relay supports the requested IPv6 connection,
    // so multihop is not required and single-hop selection should succeed.
    func testDoesNotUseMultihopWhenShadowsocksExitRelaySupportsIpv6() throws {
        let allPorts: Range<UInt16> = 1..<65000
        let defaultPortRanges = RelaySelector.parseRawPortRanges(sampleRelays.wireguard.shadowsocksPortRanges)

        let portsOutsideDefaultRange = allPorts.filter { port in
            !defaultPortRanges.contains { range in
                range.contains(port)
            }
        }
        let port = try XCTUnwrap(portsOutsideDefaultRange.randomElement())

        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "got", "se3-wireguard")])),
        )

        var settings = LatestTunnelSettings()
        settings.relayConstraints = constraints
        settings.wireGuardObfuscation = .init(state: .shadowsocks, shadowsocksPort: .custom(port))
        settings.ipVersion = .ipv6
        settings.tunnelMultihopState = .whenNeeded

        let picker = SinglehopPicker(
            relays: sampleRelays,
            tunnelSettings: settings,
            connectionAttemptCount: 0
        )

        let selectedRelays = try picker.pick()
        XCTAssertNil(selectedRelays.entry)
        XCTAssertEqual(selectedRelays.exit.hostname, "se3-wireguard")
    }
}
