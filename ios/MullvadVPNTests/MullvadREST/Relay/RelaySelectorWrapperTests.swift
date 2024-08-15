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
    var settingsUpdater: SettingsUpdater!
    var settingsListener: TunnelSettingsListener!

    override func setUp() {
        relayCache = RelayCache(fileCache: fileCache)
        settingsListener = TunnelSettingsListener()
        settingsUpdater = SettingsUpdater(listener: settingsListener)
    }

    func testSelectRelayWithMultihopOff() throws {
        let wrapper = RelaySelectorWrapper(
            relayCache: relayCache,
            tunnelSettingsUpdater: settingsUpdater
        )

        settingsListener.onNewSettings?(LatestTunnelSettings(tunnelMultihopState: .off))

        let selectedRelays = try wrapper.selectRelays(connectionAttemptCount: 0)
        XCTAssertNil(selectedRelays.entry)
    }

    func testSelectRelayWithMultihopOn() throws {
        let wrapper = RelaySelectorWrapper(
            relayCache: relayCache,
            tunnelSettingsUpdater: settingsUpdater
        )

        settingsListener.onNewSettings?(LatestTunnelSettings(tunnelMultihopState: .on))

        let selectedRelays = try wrapper.selectRelays(connectionAttemptCount: 0)
        XCTAssertNotNil(selectedRelays.entry)
    }
}
