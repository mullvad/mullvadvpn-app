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
}
