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

    func testCanSelectRelayWithMultihopOnAndDaitaOn() throws {
        let wrapper = RelaySelectorWrapper(
            relayCache: relayCache,
            tunnelSettingsUpdater: settingsUpdater
        )

        let constraints = RelayConstraints(
            entryLocations: .only(UserSelectedRelays(locations: [.country("es")])), // Relay with DAITA.
            exitLocations: .only(UserSelectedRelays(locations: [.country("us")]))
        )

        let settings = LatestTunnelSettings(
            relayConstraints: constraints,
            tunnelMultihopState: .on,
            daita: DAITASettings(state: .on)
        )
        settingsListener.onNewSettings?(settings)

        XCTAssertNoThrow(try wrapper.selectRelays(connectionAttemptCount: 0))
    }

    func testCannotSelectRelayWithMultihopOnAndDaitaOn() throws {
        let wrapper = RelaySelectorWrapper(
            relayCache: relayCache,
            tunnelSettingsUpdater: settingsUpdater
        )

        let constraints = RelayConstraints(
            entryLocations: .only(UserSelectedRelays(locations: [.country("se")])), // Relay without DAITA.
            exitLocations: .only(UserSelectedRelays(locations: [.country("us")]))
        )

        let settings = LatestTunnelSettings(
            relayConstraints: constraints,
            tunnelMultihopState: .on,
            daita: DAITASettings(state: .on)
        )
        settingsListener.onNewSettings?(settings)

        XCTAssertThrowsError(try wrapper.selectRelays(connectionAttemptCount: 0))
    }

    func testCanSelectRelayWithMultihopOffAndDaitaOn() throws {
        let wrapper = RelaySelectorWrapper(
            relayCache: relayCache,
            tunnelSettingsUpdater: settingsUpdater
        )

        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.country("es")])) // Relay with DAITA.
        )

        let settings = LatestTunnelSettings(
            relayConstraints: constraints,
            tunnelMultihopState: .off,
            daita: DAITASettings(state: .on)
        )
        settingsListener.onNewSettings?(settings)

        let selectedRelays = try wrapper.selectRelays(connectionAttemptCount: 0)
        XCTAssertNil(selectedRelays.entry)
    }

    // If DAITA is enabled and no supported relays are found, we should try to find the nearest
    // available relay that supports DAITA and use it as entry in a multihop selection.
    func testCanSelectRelayWithMultihopOffAndDaitaOnThroughMultihop() throws {
        let wrapper = RelaySelectorWrapper(
            relayCache: relayCache,
            tunnelSettingsUpdater: settingsUpdater
        )

        let constraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.country("se")])) // Relay without DAITA.
        )

        let settings = LatestTunnelSettings(
            relayConstraints: constraints,
            tunnelMultihopState: .off,
            daita: DAITASettings(state: .on)
        )
        settingsListener.onNewSettings?(settings)

        let selectedRelays = try wrapper.selectRelays(connectionAttemptCount: 0)
        XCTAssertNotNil(selectedRelays.entry)
    }
}
