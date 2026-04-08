//
//  MultihopMigrationTrackerTests.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-04-08.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadMockData
import Testing

@testable import MullvadREST
@testable import MullvadSettings
@testable import MullvadTypes

final class MultihopMigrationTrackerTests {
    var relaySelectorWrapper: RelaySelectorWrapper!
    var relayCache: RelayCache!
    init() async throws {
        let fileCache = MockFileCache(
            initialState: .exists(
                try StoredRelays(
                    rawData: try REST.Coding.makeJSONEncoder().encode(ServerRelaysResponseStubs.sampleRelays),
                    updatedAt: .distantPast
                ))
        )
        relayCache = RelayCache(fileCache: fileCache)
        relaySelectorWrapper = RelaySelectorWrapper(relayCache: relayCache)
    }

    @Test func testSetsWhenNeededWhenDaitaIsOffAndNoExitFilter() async throws {
        var tunnelSettings = LatestTunnelSettings()
        tunnelSettings.tunnelMultihopState = .never
        tunnelSettings.daita.daitaState = .off
        tunnelSettings.relayConstraints.exitFilter = .any

        let multihopMigrationTracker = MultihopMigrationTrackerFactory.make(relaySelectorWrapper)
        let output = try multihopMigrationTracker.run(input: &tunnelSettings)
        #expect(tunnelSettings.tunnelMultihopState == .whenNeeded)
        #expect(output.changes.count == 1)
        #expect(output.changes.first!.path == .updatedMultiHop)
    }
    @Test func testKeepsNeverWhenDaitaIsOffAndExitFilterIsOn() async throws {
        var tunnelSettings = LatestTunnelSettings()
        tunnelSettings.tunnelMultihopState = .never
        tunnelSettings.daita.daitaState = .off
        tunnelSettings.relayConstraints.exitFilter = .only(RelayFilter())

        let multihopMigrationTracker = MultihopMigrationTrackerFactory.make(relaySelectorWrapper)
        let output = try multihopMigrationTracker.run(input: &tunnelSettings)
        #expect(tunnelSettings.tunnelMultihopState == .never)
        #expect(output.changes.count(where: { $0.path == .updatedMultiHop || $0.path == .uniqueFilter }) == 2)
    }

    @Test func testSetsWhenNeededWhenDaitaWithSmartRoutingIsOnAndNoExitFilter() async throws {
        var tunnelSettings = LatestTunnelSettings()
        tunnelSettings.tunnelMultihopState = .never
        tunnelSettings.daita.daitaState = .on
        tunnelSettings.relayConstraints.exitFilter = .any

        let multihopMigrationTracker = MultihopMigrationTrackerFactory.make(relaySelectorWrapper)
        let output = try multihopMigrationTracker.run(input: &tunnelSettings)
        #expect(tunnelSettings.tunnelMultihopState == .whenNeeded)
        #expect(output.changes.count(where: { $0.path == .updatedMultiHop }) == 1)
    }

    @Test func testKeepsNeverWhenDaitaWithSmartRoutingIsOnAndExitFilterIsOnButNotNeeded() async throws {
        var tunnelSettings = LatestTunnelSettings()
        tunnelSettings.tunnelMultihopState = .never
        tunnelSettings.daita.daitaState = .on
        tunnelSettings.relayConstraints.exitFilter = .only(RelayFilter(ownership: .rented, providers: .only(["100TB"])))
        tunnelSettings.relayConstraints.exitLocations = .only(UserSelectedRelays(locations: [.city("es", "mad")]))

        let multihopMigrationTracker = MultihopMigrationTrackerFactory.make(relaySelectorWrapper)
        let output = try multihopMigrationTracker.run(input: &tunnelSettings)
        #expect(tunnelSettings.tunnelMultihopState == .never)
        #expect(output.changes.count(where: { $0.path == .updatedMultiHop || $0.path == .uniqueFilter }) == 2)
    }

    @Test func testSetsAlwaysWhenDaitaWithSmartRoutingIsOnAndExitFilterIsOnAndNeeded() async throws {
        var tunnelSettings = LatestTunnelSettings()
        tunnelSettings.tunnelMultihopState = .never
        tunnelSettings.daita.daitaState = .on
        tunnelSettings.relayConstraints.exitFilter = .only(RelayFilter(ownership: .rented, providers: .only(["Blix"])))
        tunnelSettings.relayConstraints.exitLocations = .only(UserSelectedRelays(locations: [.city("se", "got")]))

        let multihopMigrationTracker = MultihopMigrationTrackerFactory.make(relaySelectorWrapper)
        let output = try multihopMigrationTracker.run(input: &tunnelSettings)
        #expect(tunnelSettings.tunnelMultihopState == .always)
        #expect(output.changes.count(where: { $0.path == .updatedMultiHop || $0.path == .uniqueFilter }) == 2)
        #expect(output.action != nil)
    }

    @Test func testKeepsNeverWhenDirectOnlyIsEnabledAndNoExitFilter() async throws {
        var tunnelSettings = LatestTunnelSettings()
        tunnelSettings.tunnelMultihopState = .never
        tunnelSettings.daita.daitaState = .on
        tunnelSettings.daita.directOnlyState = .on

        let multihopMigrationTracker = MultihopMigrationTrackerFactory.make(relaySelectorWrapper)
        let output = try multihopMigrationTracker.run(input: &tunnelSettings)
        #expect(tunnelSettings.tunnelMultihopState == .never)
        #expect(output.changes.count(where: { $0.path == .updatedMultiHop || $0.path == .directOnlyRemoved }) == 2)
    }

    @Test func testKeepsNeverWhenDirectOnlyIsEnabledAndExitFilterIsOn() async throws {
        var tunnelSettings = LatestTunnelSettings()
        tunnelSettings.tunnelMultihopState = .never
        tunnelSettings.daita.daitaState = .on
        tunnelSettings.daita.directOnlyState = .on

        let multihopMigrationTracker = MultihopMigrationTrackerFactory.make(relaySelectorWrapper)
        let output = try multihopMigrationTracker.run(input: &tunnelSettings)
        #expect(tunnelSettings.tunnelMultihopState == .never)
        #expect(output.changes.count(where: { $0.path == .updatedMultiHop || $0.path == .directOnlyRemoved }) == 2)
    }

    @Test func testKeepsAlwaysWhenDaitaIsOffAndNoEntryFilter() async throws {
        var tunnelSettings = LatestTunnelSettings()
        tunnelSettings.tunnelMultihopState = .always
        tunnelSettings.daita.daitaState = .off
        tunnelSettings.relayConstraints.entryFilter = .any

        let multihopMigrationTracker = MultihopMigrationTrackerFactory.make(relaySelectorWrapper)
        let output = try multihopMigrationTracker.run(input: &tunnelSettings)
        #expect(tunnelSettings.tunnelMultihopState == .always)
        #expect(output.changes.count(where: { $0.path == .updatedMultiHop }) == 1)
    }

    @Test func testKeepsAlwaysWhenDaitaIsOffAndEntryFilterIsOn() async throws {
        var tunnelSettings = LatestTunnelSettings()
        tunnelSettings.tunnelMultihopState = .always
        tunnelSettings.daita.daitaState = .off
        tunnelSettings.relayConstraints.entryFilter = .only(RelayFilter(ownership: .owned, providers: .only(["Blix"])))

        let multihopMigrationTracker = MultihopMigrationTrackerFactory.make(relaySelectorWrapper)
        let output = try multihopMigrationTracker.run(input: &tunnelSettings)
        #expect(tunnelSettings.tunnelMultihopState == .always)
        #expect(output.changes.count(where: { $0.path == .updatedMultiHop || $0.path == .uniqueFilter }) == 2)
    }

    @Test func testKeepsAlwaysWhenDaitaIsOnAndNoEntryFilter() async throws {
        var tunnelSettings = LatestTunnelSettings()
        tunnelSettings.tunnelMultihopState = .always
        tunnelSettings.daita.daitaState = .on
        tunnelSettings.daita.directOnlyState = .off
        tunnelSettings.relayConstraints.entryFilter = .any

        let multihopMigrationTracker = MultihopMigrationTrackerFactory.make(relaySelectorWrapper)
        let output = try multihopMigrationTracker.run(input: &tunnelSettings)
        #expect(tunnelSettings.tunnelMultihopState == .always)
        #expect(output.changes.count(where: { $0.path == .updatedMultiHop }) == 1)
    }

    @Test func testKeepsAlwaysWhenDaitaIsOnAndEntryFilterIsOn() async throws {
        var tunnelSettings = LatestTunnelSettings()
        tunnelSettings.tunnelMultihopState = .always
        tunnelSettings.daita.daitaState = .on
        tunnelSettings.daita.directOnlyState = .off
        tunnelSettings.relayConstraints.entryFilter = .only(RelayFilter(ownership: .owned, providers: .only(["Blix"])))

        let multihopMigrationTracker = MultihopMigrationTrackerFactory.make(relaySelectorWrapper)
        let output = try multihopMigrationTracker.run(input: &tunnelSettings)
        #expect(tunnelSettings.tunnelMultihopState == .always)
        #expect(output.changes.count(where: { $0.path == .updatedMultiHop || $0.path == .uniqueFilter }) == 2)
        #expect(output.action != nil)
    }

    @Test func testKeepsAlwaysWhenDaitaWithDirectOnlyIsOnAndNoEntryFilter() async throws {
        var tunnelSettings = LatestTunnelSettings()
        tunnelSettings.tunnelMultihopState = .always
        tunnelSettings.daita.daitaState = .on
        tunnelSettings.daita.directOnlyState = .on
        tunnelSettings.relayConstraints.entryFilter = .any

        let multihopMigrationTracker = MultihopMigrationTrackerFactory.make(relaySelectorWrapper)
        let output = try multihopMigrationTracker.run(input: &tunnelSettings)
        #expect(tunnelSettings.tunnelMultihopState == .always)
        #expect(output.changes.count(where: { $0.path == .updatedMultiHop || $0.path == .directOnlyRemoved }) == 2)
    }

    @Test func testKeepsAlwaysWhenDaitaWithDirectOnlyIsOnAndEntryFilterIsOn() async throws {
        var tunnelSettings = LatestTunnelSettings()
        tunnelSettings.tunnelMultihopState = .always
        tunnelSettings.daita.daitaState = .on
        tunnelSettings.daita.directOnlyState = .on
        tunnelSettings.relayConstraints.entryFilter = .only(RelayFilter(ownership: .owned, providers: .only(["Blix"])))

        let multihopMigrationTracker = MultihopMigrationTrackerFactory.make(relaySelectorWrapper)
        let output = try multihopMigrationTracker.run(input: &tunnelSettings)
        #expect(tunnelSettings.tunnelMultihopState == .always)
        #expect(
            output.changes.count(where: {
                $0.path == .updatedMultiHop || $0.path == .uniqueFilter || $0.path == .directOnlyRemoved
            }) == 3)
        #expect(output.action != nil)
    }
}
