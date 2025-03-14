//
//  RelaySelectorWrapperTests.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2024-06-10.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadMockData
@testable import MullvadREST
@testable import MullvadSettings
@testable import MullvadTypes
import XCTest

class RelaySelectorWrapperTests: XCTestCase {
    let multihopWithDaitaConstraints = RelayConstraints(
        entryLocations: .only(UserSelectedRelays(locations: [.country("es")])), // Relay with DAITA.
        exitLocations: .only(UserSelectedRelays(locations: [.country("us")]))
    )

    let multihopWithoutDaitaConstraints = RelayConstraints(
        entryLocations: .only(UserSelectedRelays(locations: [.country("se")])), // Relay without DAITA.
        exitLocations: .only(UserSelectedRelays(locations: [.country("us")]))
    )

    let singlehopWithoutDaitaConstraints = RelayConstraints(
        exitLocations: .only(UserSelectedRelays(locations: [.country("se")])) // Relay without DAITA.
    )

    let singlehopWithDaitaConstraints = RelayConstraints(
        exitLocations: .only(UserSelectedRelays(locations: [.country("es")])) // Relay with DAITA.
    )

    var relayCache: RelayCache!
    override func setUpWithError() throws {
        let fileCache = MockFileCache(
            initialState: .exists(try StoredRelays(
                rawData: try REST.Coding.makeJSONEncoder().encode(ServerRelaysResponseStubs.sampleRelays),
                updatedAt: .distantPast
            ))
        )

        relayCache = RelayCache(fileCache: fileCache)
    }

    func testSelectRelayWithMultihopOff() throws {
        let wrapper = RelaySelectorWrapper(relayCache: relayCache)

        let settings = LatestTunnelSettings(
            relayConstraints: singlehopWithoutDaitaConstraints,
            tunnelMultihopState: .off,
            daita: DAITASettings(daitaState: .off)
        )

        let selectedRelays = try wrapper.selectRelays(tunnelSettings: settings, connectionAttemptCount: 0)
        XCTAssertNil(selectedRelays.entry)
    }

    func testSelectRelayWithMultihopOn() throws {
        let wrapper = RelaySelectorWrapper(relayCache: relayCache)

        let settings = LatestTunnelSettings(
            relayConstraints: multihopWithDaitaConstraints,
            tunnelMultihopState: .on,
            daita: DAITASettings(daitaState: .off)
        )

        let selectedRelays = try wrapper.selectRelays(tunnelSettings: settings, connectionAttemptCount: 0)
        XCTAssertNotNil(selectedRelays.entry)
    }

    func testCanSelectRelayWithMultihopOnAndDaitaOn() throws {
        let wrapper = RelaySelectorWrapper(relayCache: relayCache)

        let settings = LatestTunnelSettings(
            relayConstraints: multihopWithDaitaConstraints,
            tunnelMultihopState: .on,
            daita: DAITASettings(daitaState: .on)
        )

        XCTAssertNoThrow(try wrapper.selectRelays(tunnelSettings: settings, connectionAttemptCount: 0))
    }

    func testCannotSelectRelayWithMultihopOnDaitaOnDirectOnlyOn() throws {
        let wrapper = RelaySelectorWrapper(relayCache: relayCache)

        let settings = LatestTunnelSettings(
            relayConstraints: multihopWithoutDaitaConstraints,
            tunnelMultihopState: .on,
            daita: DAITASettings(daitaState: .on, directOnlyState: .on)
        )

        XCTAssertThrowsError(try wrapper.selectRelays(tunnelSettings: settings, connectionAttemptCount: 0))
    }

    func testCanSelectRelayWithMultihopOffAndDaitaOn() throws {
        let wrapper = RelaySelectorWrapper(relayCache: relayCache)

        let settings = LatestTunnelSettings(
            relayConstraints: singlehopWithDaitaConstraints,
            tunnelMultihopState: .off,
            daita: DAITASettings(daitaState: .on)
        )

        let selectedRelays = try wrapper.selectRelays(tunnelSettings: settings, connectionAttemptCount: 0)
        XCTAssertNotNil(selectedRelays.exit)
    }

    // If DAITA is enabled and no supported relays are found, we should try to find the nearest
    // available relay that supports DAITA and use it as entry in a multihop selection.
    func testCanSelectRelayWithMultihopOffDaitaOnThroughMultihop() throws {
        let wrapper = RelaySelectorWrapper(relayCache: relayCache)

        let settings = LatestTunnelSettings(
            relayConstraints: singlehopWithoutDaitaConstraints,
            tunnelMultihopState: .off,
            daita: DAITASettings(daitaState: .on)
        )

        let selectedRelays = try wrapper.selectRelays(tunnelSettings: settings, connectionAttemptCount: 0)
        XCTAssertNotNil(selectedRelays.entry)
    }
}
