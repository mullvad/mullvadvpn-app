//
//  RecentConnectionRepositoryTest.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-09-25.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
@testable import MullvadSettings
@testable import MullvadTypes
import Testing

@Suite("RecentConnectionRepositoryTest")
class RecentConnectionRepositoryTest {
    @Test("Return the most recent connections with a 50 limit by default", arguments: [
        [RecentConnection(
            entry: UserSelectedRelays(locations: [.country("se")]),
            exit: UserSelectedRelays(locations: [.city("al", "tia")])
        )],
    ])
    func add(recentConnections: [RecentConnection]) async {
        let store = InMemorySettingsStore<SettingNotFound>()
        store.reset()

        SettingsManager.unitTestStore = store

        let repository = RecentConnectionRepository()
        for recentConnection in recentConnections {
            await repository.add(recentConnection)
        }

        let recentConnections = await repository.all()
        #expect(recentConnections.count == recentConnections.count)
    }

    @Test("Return the most recent connections with a 2 limit")
    func exceedLimit() async {
        let store = InMemorySettingsStore<SettingNotFound>()
        store.reset()

        SettingsManager.unitTestStore = store

        let repository = RecentConnectionRepository()
        await repository.clear()

        let maxLimit = 2
        let params: [RecentConnection] = [RecentConnection(
            entry: UserSelectedRelays(locations: [.country("se")]),
            exit: UserSelectedRelays(locations: [.city("al", "tia")])
        ), RecentConnection(
            entry: UserSelectedRelays(locations: [.country("se")]),
            exit: UserSelectedRelays(locations: [.country("fr")])
        ), RecentConnection(
            entry: UserSelectedRelays(locations: [.country("de")]),
            exit: UserSelectedRelays(locations: [.country("nl")])
        )]
        for recentConnection in params {
            await repository.add(recentConnection, maxLimit: maxLimit)
        }

        let recentConnections = await repository.all()
        #expect(recentConnections.count == maxLimit)
        #expect(recentConnections.contains(params.first!) == false)
    }

    @Test("Should not store a duplicate RecentConnection")
    func addDuplicate() async {
        let params: [RecentConnection] = [RecentConnection(
            exit: UserSelectedRelays(locations: [.country("se")])
        ), RecentConnection(
            exit: UserSelectedRelays(locations: [.country("sp")])
        ), RecentConnection(
            exit: UserSelectedRelays(locations: [.country("se")])
        ), RecentConnection(
            entry: UserSelectedRelays(locations: [.country("se")]),
            exit: UserSelectedRelays(locations: [.country("nl")])
        )]

        let store = InMemorySettingsStore<SettingNotFound>()
        store.reset()

        SettingsManager.unitTestStore = store

        let repository = RecentConnectionRepository()
        for recentConnection in params {
            await repository.add(recentConnection)
        }

        let recentConnections = await repository.all()
        #expect(recentConnections.count == params.count - 1)
    }

    @Test("Clear all RecentConnections")
    func cleanup() async throws {
        let store = InMemorySettingsStore<SettingNotFound>()
        store.reset()

        SettingsManager.unitTestStore = store

        let repository = RecentConnectionRepository()
        await repository.clear()
        let recentConnections = await repository.all()
        #expect(recentConnections.isEmpty)
    }
}
