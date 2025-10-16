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

    @Test("Adds items up to the limit 1 for either entry or exit")
    func addLocations() throws {
        let maxLimit = 1
        let repository = makeRepository(max: maxLimit)
        try repository.setRecentsEnabled(true)
        try addLocations(repository, locations: [se, de], to: .entry)
        try addLocations(repository, locations: [de], to: .exit)

        let recentsSettings = try repository.all()
        #expect(recentsSettings.isEnabled)
        #expect(recentsSettings.exitLocations.count == maxLimit)
        #expect(recentsSettings.entryLocations.count == maxLimit)
    }

    @Test("Adds items up to the default limit (50) for either entry or exit")
    func addDuplicate() throws {
        let repository = makeRepository()
        try repository.setRecentsEnabled(true)
        try addLocations(repository, locations: [se, de], to: .entry)
        try addLocations(repository, locations: [de, se, nl, se], to: .exit)

        let recentsSettings = try repository.all()
        #expect(recentsSettings.isEnabled)
        #expect(recentsSettings.entryLocations.count == 2)
        #expect(recentsSettings.exitLocations.count == 3)
    }

    @Test("Removes all recents settings with disabling recents")
    func disable() throws {
        let repository = makeRepository()
        try repository.setRecentsEnabled(true)
        try addLocations(repository, locations: [se, de], to: .entry)
        try addLocations(repository, locations: [de, se, nl], to: .exit)

        var recentsSettings = try repository.all()
        #expect(recentsSettings.isEnabled)
        #expect(recentsSettings.entryLocations.count == 2)
        #expect(recentsSettings.exitLocations.count == 3)

        try repository.setRecentsEnabled(false)

        recentsSettings = try repository.all()
        #expect(!recentsSettings.isEnabled)
        #expect(recentsSettings.entryLocations.count == 0)
        #expect(recentsSettings.exitLocations.count == 0)

    }

    @Test("Tries to add location before enabling recents")
    func addRecentsBeforeEnablingRecents() throws {
        let repository = makeRepository()

        try repository.setRecentsEnabled(false)
        let action: () throws -> Void = { [self] in
            try addLocations(
                repository,
                locations: [self.se, self.de],
                to: RecentLocationType.entry
            )
        }

        #expect(throws: RecentConnectionsRepositoryError.recentsDisabled, performing: action)
    }

    private func makeRepository(max: Int = 50) -> RecentConnectionsRepository {
        return RecentConnectionsRepository(store: InMemorySettingsStore<SettingNotFound>(), maxLimit: max)
    }

    private func addLocations(
        _ repository: RecentConnectionsRepository, locations: [UserSelectedRelays], to: RecentLocationType
    ) throws {
        for location in locations {
            try repository.add(location, to: to)
        }
    }
}
