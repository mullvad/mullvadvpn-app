//
//  RelaySelectorWrapperTests.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2024-06-10.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadREST
@testable import MullvadSettings
@testable import MullvadTypes
import XCTest

class RelaySelectorWrapperTests: XCTestCase {
    let fileCache = MockFileCache(
        initialState: .exists(CachedRelays(
            relays: ServerRelaysResponseStubs.sampleRelays,
            updatedAt: .distantPast
        ))
    )
    let multihopWithDaitaConstraints = RelayConstraints(
        entryLocations: .only(UserSelectedRelays(locations: [.country("es")])), // Relay with DAITA.
        exitLocations: .only(UserSelectedRelays(locations: [.country("us")]))
    )

    let multihopWithoutDaitaConstraints = RelayConstraints(
        entryLocations: .only(UserSelectedRelays(locations: [.country("se")])), // Relay without DAITA.
        exitLocations: .only(UserSelectedRelays(locations: [.country("us")]))
    )

    let singlehopConstraints = RelayConstraints(
        exitLocations: .only(UserSelectedRelays(locations: [.country("se")])) // Relay without DAITA.
    )

    var relayCache: RelayCache!
    override func setUp() {
        relayCache = RelayCache(fileCache: fileCache)
    }

    func testSelectRelayWithMultihopOff() throws {
        let wrapper = RelaySelectorWrapper(relayCache: relayCache)

        let settings = LatestTunnelSettings(
            relayConstraints: singlehopConstraints,
            tunnelMultihopState: .off,
            daita: DAITASettings(state: .off)
        )

        let selectedRelays = try wrapper.selectRelays(tunnelSettings: settings, connectionAttemptCount: 0)
        XCTAssertNil(selectedRelays.entry)
    }

    func testSelectRelayWithMultihopOn() throws {
        let wrapper = RelaySelectorWrapper(relayCache: relayCache)

        let settings = LatestTunnelSettings(
            relayConstraints: multihopWithDaitaConstraints,
            tunnelMultihopState: .on,
            daita: DAITASettings(state: .off)
        )

        let selectedRelays = try wrapper.selectRelays(tunnelSettings: settings, connectionAttemptCount: 0)
        XCTAssertNotNil(selectedRelays.entry)
    }

    func testCanSelectRelayWithMultihopOnAndDaitaOn() throws {
        let wrapper = RelaySelectorWrapper(relayCache: relayCache)

        let settings = LatestTunnelSettings(
            relayConstraints: multihopWithDaitaConstraints,
            tunnelMultihopState: .on,
            daita: DAITASettings(state: .on)
        )

        XCTAssertNoThrow(try wrapper.selectRelays(tunnelSettings: settings, connectionAttemptCount: 0))
    }

    func testCannotSelectRelayWithMultihopOnAndDaitaOn() throws {
        let wrapper = RelaySelectorWrapper(relayCache: relayCache)

        let settings = LatestTunnelSettings(
            relayConstraints: multihopWithoutDaitaConstraints,
            tunnelMultihopState: .on,
            daita: DAITASettings(state: .on)
        )

        XCTAssertThrowsError(try wrapper.selectRelays(tunnelSettings: settings, connectionAttemptCount: 0))
    }

    func testCanSelectRelayWithMultihopOffAndDaitaOn() throws {
        let wrapper = RelaySelectorWrapper(relayCache: relayCache)

        let settings = LatestTunnelSettings(
            relayConstraints: multihopWithoutDaitaConstraints,
            tunnelMultihopState: .off,
            daita: DAITASettings(state: .on)
        )

        let selectedRelays = try wrapper.selectRelays(tunnelSettings: settings, connectionAttemptCount: 0)
        XCTAssertNil(selectedRelays.entry)
    }

    // If DAITA is enabled and no supported relays are found, we should try to find the nearest
    // available relay that supports DAITA and use it as entry in a multihop selection.
    func testCanSelectRelayWithMultihopOffAndDaitaOnThroughMultihop() throws {
        let wrapper = RelaySelectorWrapper(relayCache: relayCache)

        let settings = LatestTunnelSettings(
            relayConstraints: singlehopConstraints,
            tunnelMultihopState: .off,
            daita: DAITASettings(state: .on)
        )

        let selectedRelays = try wrapper.selectRelays(tunnelSettings: settings, connectionAttemptCount: 0)
        XCTAssertNotNil(selectedRelays.entry)
    }
}
