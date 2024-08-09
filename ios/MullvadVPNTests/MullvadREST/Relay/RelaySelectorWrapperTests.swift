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

    var relayCache: RelayCache!
    var multihopUpdater: MultihopUpdater!
    var multihopStateListener: MultihopStateListener!

    override func setUp() {
        relayCache = RelayCache(fileCache: fileCache)
        multihopStateListener = MultihopStateListener()
        multihopUpdater = MultihopUpdater(listener: multihopStateListener)
    }

    func testSelectRelayWithMultihopOff() throws {
        let wrapper = RelaySelectorWrapper(
            relayCache: relayCache,
            multihopUpdater: multihopUpdater
        )

        multihopStateListener.onNewMultihop?(.off)

        let selectedRelays = try wrapper.selectRelays(with: RelayConstraints(), connectionAttemptCount: 0)
        XCTAssertNil(selectedRelays.entry)
    }

    func testSelectRelayWithMultihopOn() throws {
        let wrapper = RelaySelectorWrapper(
            relayCache: relayCache,
            multihopUpdater: multihopUpdater
        )

        multihopStateListener.onNewMultihop?(.on)

        let selectedRelays = try wrapper.selectRelays(with: RelayConstraints(), connectionAttemptCount: 0)
        XCTAssertNotNil(selectedRelays.entry)
    }

    func testCanSelectRelayWithMultihopOnAndDaitaOn() throws {
        let wrapper = RelaySelectorWrapper(
            relayCache: relayCache,
            multihopUpdater: multihopUpdater
        )

        multihopStateListener.onNewMultihop?(.on)
        wrapper.setDaita(state: .on)

        let constraints = RelayConstraints(
            entryLocations: .only(UserSelectedRelays(locations: [.country("es")])), // Relay with DAITA.
            exitLocations: .only(UserSelectedRelays(locations: [.country("us")]))
        )

        XCTAssertNoThrow(try wrapper.selectRelays(with: constraints, connectionAttemptCount: 0))
    }

    func testCannotSelectRelayWithMultihopOnAndDaitaOn() throws {
        let wrapper = RelaySelectorWrapper(
            relayCache: relayCache,
            multihopUpdater: multihopUpdater
        )

        multihopStateListener.onNewMultihop?(.on)
        wrapper.setDaita(state: .on)

        let constraints = RelayConstraints(
            entryLocations: .only(UserSelectedRelays(locations: [.country("se")])), // Relay without DAITA.
            exitLocations: .only(UserSelectedRelays(locations: [.country("us")]))
        )

        XCTAssertThrowsError(try wrapper.selectRelays(with: constraints, connectionAttemptCount: 0))
    }

    func testCanSelectRelayWithMultihopOffAndDaitaOn() throws {
        let wrapper = RelaySelectorWrapper(
            relayCache: relayCache,
            multihopUpdater: multihopUpdater
        )

        multihopStateListener.onNewMultihop?(.off)
        wrapper.setDaita(state: .on)

        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.country("es")])) // Relay with DAITA.
        )

        let selectedRelays = try wrapper.selectRelays(with: constraints, connectionAttemptCount: 0)
        XCTAssertNil(selectedRelays.entry)
    }

    // If DAITA is enabled and no supported relays are found, we should try to find the nearest
    // available relay that supports DAITA and use it as entry in a multihop selection.
    func testCanSelectRelayWithMultihopOffAndDaitaOnThroughMultihop() throws {
        let wrapper = RelaySelectorWrapper(
            relayCache: relayCache,
            multihopUpdater: multihopUpdater
        )

        multihopStateListener.onNewMultihop?(.off)
        wrapper.setDaita(state: .on)

        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.country("se")])) // Relay without DAITA.
        )

        let selectedRelays = try wrapper.selectRelays(with: constraints, connectionAttemptCount: 0)
        XCTAssertNotNil(selectedRelays.entry)
    }
}
