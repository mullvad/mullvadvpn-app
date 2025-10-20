//
//  RecentConnectionsRepositoryTests.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-09-25.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import Testing

@testable import MullvadSettings
@testable import MullvadTypes

@Suite("RecentConnectionsRepositoryTests")
final class RecentConnectionsRepositoryTests {
    let se = UserSelectedRelays(locations: [.country("se")])
    let fr = UserSelectedRelays(locations: [.country("fr")])
    let nl = UserSelectedRelays(locations: [.country("nl")])
    let de = UserSelectedRelays(locations: [.country("de")])

    @Test("Adds locations up to the limit 1 for either entry or exit")
    func addLocations() throws {
        let maxLimit: UInt = 1
        let repository = makeRepository(max: maxLimit)
        try repository.setRecentsEnabled(true)
        try addLocations(repository, locations: [se, de], as: .entry)
        try addLocations(repository, locations: [de], as: .exit)

        let recentsSettings = try repository.all()
        #expect(recentsSettings.isEnabled)
        #expect(recentsSettings.exitLocations.count == maxLimit)
        #expect(recentsSettings.entryLocations.count == maxLimit)
    }

    @Test("Adds locations up to the default limit (50) for either entry or exit")
    func addDuplicate() throws {
        let repository = makeRepository()
        try repository.setRecentsEnabled(true)
        try addLocations(repository, locations: [se, de], as: .entry)
        try addLocations(repository, locations: [de, se, nl, se], as: .exit)

        let recentsSettings = try repository.all()
        #expect(recentsSettings.isEnabled)
        #expect(recentsSettings.entryLocations.count == 2)
        #expect(recentsSettings.exitLocations.count == 3)
    }

    @Test("Removes all recents connections with disabling recents")
    func disable() throws {
        let repository = makeRepository()
        try repository.setRecentsEnabled(true)
        try addLocations(repository, locations: [se, de], as: .entry)
        try addLocations(repository, locations: [de, se, nl], as: .exit)

        var recentConnections = try repository.all()
        #expect(recentConnections.isEnabled)
        #expect(recentConnections.entryLocations.count == 2)
        #expect(recentConnections.exitLocations.count == 3)

        try repository.setRecentsEnabled(false)

        recentConnections = try repository.all()
        #expect(!recentConnections.isEnabled)
        #expect(recentConnections.entryLocations.count == 0)
        #expect(recentConnections.exitLocations.count == 0)

    }

    @Test("Fails with an error if a location is added while recents are disabled.")
    func addRecentsBeforeEnablingRecents() throws {
        let repository = makeRepository()

        try repository.setRecentsEnabled(false)
        let action: () throws -> Void = { [self] in
            try addLocations(
                repository,
                locations: [self.se],
                as: RecentLocationType.entry
            )
        }

        #expect(throws: RecentConnectionsRepositoryError.recentsDisabled, performing: action)
    }

    private func makeRepository(max: UInt = 50) -> RecentConnectionsRepository {
        return RecentConnectionsRepository(store: InMemorySettingsStore<SettingNotFound>(), maxLimit: max)
    }

    private func addLocations(
        _ repository: RecentConnectionsRepository, locations: [UserSelectedRelays], as type: RecentLocationType
    ) throws {
        for location in locations {
            try repository.add(location, as: type)
        }
    }
}
