//
//  MultihopValidatorTests.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-05-21.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadMockData
import XCTest

@testable import MullvadREST
@testable import MullvadSettings
@testable import MullvadTypes

class MultihopValidatorTests: XCTestCase {
    let multihopWithFiltersConstraints = RelayConstraints(
        entryFilter: .only(.init(ownership: .owned)),
        exitFilter: .only(.init(ownership: .owned))
    )

    let singlehopWithFiltersConstraints = RelayConstraints(
        exitFilter: .only(.init(ownership: .owned))
    )

    let multihopWithoutDaitaConstraints = RelayConstraints(
        entryLocations: .only(UserSelectedRelays(locations: [.country("se")])),
        exitLocations: .only(UserSelectedRelays(locations: [.country("se")]))
    )

    let singlehopWithoutDaitaConstraints = RelayConstraints(
        exitLocations: .only(UserSelectedRelays(locations: [.country("se")]))
    )

    var relaySelector: RelaySelectorWrapper!

    override func setUpWithError() throws {
        let fileCache = MockFileCache(
            initialState: .exists(
                try StoredRelays(
                    rawData: try REST.Coding.makeJSONEncoder().encode(ServerRelaysResponseStubs.sampleRelays),
                    updatedAt: .distantPast
                ))
        )

        relaySelector = RelaySelectorWrapper(relayCache: RelayCache(fileCache: fileCache))
    }

    // MARK: Filters with multihop state

    func testSelectRelayWithMultihopWhenNeededAndFiltersWillNotOverrideIfAlreadyOverriding() throws {
        let settings = LatestTunnelSettings(
            relayConstraints: multihopWithFiltersConstraints,
            tunnelMultihopState: .whenNeeded
        )

        let validator = MultihopValidator(tunnelSettings: settings, relaySelector: relaySelector)
        XCTAssertFalse(validator.stateWillOverrideFilters(.whenNeeded))
    }

    func testSelectRelayWithSinglehopAndFiltersWillNotOverride() throws {
        let settings = LatestTunnelSettings(
            relayConstraints: singlehopWithFiltersConstraints
        )

        let validator = MultihopValidator(tunnelSettings: settings, relaySelector: relaySelector)
        XCTAssertFalse(validator.stateWillOverrideFilters(.never))
    }

    func testSelectRelayWithMultihopAlwaysAndFiltersWillNotOverride() throws {
        let settings = LatestTunnelSettings(
            relayConstraints: multihopWithFiltersConstraints
        )

        let validator = MultihopValidator(tunnelSettings: settings, relaySelector: relaySelector)
        XCTAssertFalse(validator.stateWillOverrideFilters(.always))
    }

    func testSelectRelayWithMultihopWhenNeededAndFiltersWillOverride() throws {
        let settings = LatestTunnelSettings(
            relayConstraints: multihopWithFiltersConstraints
        )

        let validator = MultihopValidator(tunnelSettings: settings, relaySelector: relaySelector)
        XCTAssertTrue(validator.stateWillOverrideFilters(.whenNeeded))
    }

    func testSelectRelayWithultihopAlwaysAndAutomaticLocationAndFiltersWillOverride() throws {
        var constraints = multihopWithFiltersConstraints
        constraints.entryLocations = .any

        let settings = LatestTunnelSettings(
            relayConstraints: constraints
        )

        let validator = MultihopValidator(tunnelSettings: settings, relaySelector: relaySelector)
        XCTAssertTrue(validator.stateWillOverrideFilters(.always))
    }

    // MARK: Filters with location

    func testSelectAutomaticEntryRelayWithFiltersWillNotOverrideIfAlreadyOverriding() throws {
        let settings = LatestTunnelSettings(
            relayConstraints: multihopWithFiltersConstraints,
            tunnelMultihopState: .whenNeeded
        )

        let validator = MultihopValidator(tunnelSettings: settings, relaySelector: relaySelector)
        XCTAssertFalse(validator.locationWillOverrideFilters(AutomaticLocationNode(), context: .entry))
    }

    func testSelectExitRelayWithFiltersWillNotOverride() throws {
        let settings = LatestTunnelSettings(
            relayConstraints: multihopWithFiltersConstraints
        )

        let validator = MultihopValidator(tunnelSettings: settings, relaySelector: relaySelector)
        XCTAssertFalse(
            validator.locationWillOverrideFilters(LocationNode(name: "Stockholm", code: "se-sto"), context: .exit)
        )
    }

    func testSelectEntryRelayWithFiltersWillNotOverride() throws {
        let settings = LatestTunnelSettings(
            relayConstraints: multihopWithFiltersConstraints
        )

        let validator = MultihopValidator(tunnelSettings: settings, relaySelector: relaySelector)
        XCTAssertFalse(
            validator.locationWillOverrideFilters(LocationNode(name: "Stockholm", code: "se-sto"), context: .entry)
        )
    }

    func testSelectAutomaticEntryRelayWithFiltersWillOverride() throws {
        let settings = LatestTunnelSettings(
            relayConstraints: multihopWithFiltersConstraints
        )

        let validator = MultihopValidator(tunnelSettings: settings, relaySelector: relaySelector)
        XCTAssertTrue(validator.locationWillOverrideFilters(AutomaticLocationNode(), context: .entry))
    }

    // MARK: Blocked state

    func testSelectRelayWithSinglehopIsCompatible() throws {
        let settings = LatestTunnelSettings(
            relayConstraints: singlehopWithoutDaitaConstraints,
            daita: DAITASettings(daitaState: .off)
        )

        let validator = MultihopValidator(tunnelSettings: settings, relaySelector: relaySelector)
        XCTAssertFalse(validator.stateIsIncompatible(.never))
    }

    func testSelectRelayWithSinglehopIsNotCompatible() throws {
        let settings = LatestTunnelSettings(
            relayConstraints: singlehopWithoutDaitaConstraints,
            daita: DAITASettings(daitaState: .on)
        )

        let validator = MultihopValidator(tunnelSettings: settings, relaySelector: relaySelector)
        XCTAssertTrue(validator.stateIsIncompatible(.never))
    }

    func testSelectRelayWithMultihopAlwaysIsCompatible() throws {
        let settings = LatestTunnelSettings(
            relayConstraints: multihopWithoutDaitaConstraints,
            daita: DAITASettings(daitaState: .off)
        )

        let validator = MultihopValidator(tunnelSettings: settings, relaySelector: relaySelector)
        XCTAssertFalse(validator.stateIsIncompatible(.always))
    }

    func testSelectRelayWithMultihopAlwaysIsNotCompatible() throws {
        let settings = LatestTunnelSettings(
            relayConstraints: multihopWithoutDaitaConstraints,
            daita: DAITASettings(daitaState: .on)
        )

        let validator = MultihopValidator(tunnelSettings: settings, relaySelector: relaySelector)
        XCTAssertTrue(validator.stateIsIncompatible(.always))
    }

    func testSelectRelayWithMultihopAlwaysAndAutomaticLocationIsCompatible() throws {
        var constraints = multihopWithoutDaitaConstraints
        constraints.entryLocations = .any

        let settings = LatestTunnelSettings(
            relayConstraints: constraints,
            daita: DAITASettings(daitaState: .on)
        )

        let validator = MultihopValidator(tunnelSettings: settings, relaySelector: relaySelector)
        XCTAssertFalse(validator.stateIsIncompatible(.always))
    }

    func testSelectRelayWithMultihopWhenNeededIsCompatible() throws {
        let settings = LatestTunnelSettings(
            relayConstraints: multihopWithoutDaitaConstraints,
            daita: DAITASettings(daitaState: .on)
        )

        let validator = MultihopValidator(tunnelSettings: settings, relaySelector: relaySelector)
        XCTAssertFalse(validator.stateIsIncompatible(.whenNeeded))
    }
}
