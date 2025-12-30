//
//  RecentConnectionsRepositoryTests.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-09-25.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Combine
import Testing

@testable import MullvadSettings
@testable import MullvadTypes

@Suite("RecentConnectionsRepositoryTests")
final class RecentConnectionsRepositoryTests {
    let se = UserSelectedRelays(locations: [.country("se")])
    let fr = UserSelectedRelays(locations: [.country("fr")])
    let nl = UserSelectedRelays(locations: [.country("nl")])
    let de = UserSelectedRelays(locations: [.country("de")])
    private var cancellables = Set<Combine.AnyCancellable>()

    @Test("Adds locations up to the limit 1 for either entry or exit")
    func addLocations() throws {
        let maxLimit: UInt = 1
        let repository = makeRepository(max: maxLimit)
        var recentConnections: RecentConnections?
        var thrownError: Error?

        repository
            .recentConnectionsPublisher
            .sink(receiveValue: { result in
                switch result {
                case .success(let value):
                    recentConnections = value
                case .failure(let error):
                    thrownError = error
                }
            })
            .store(in: &cancellables)

        repository.enable(se, selectedExitRelays: de)
        repository.add(de, selectedExitRelays: se)

        let value = try #require(recentConnections)
        #expect(thrownError == nil)
        #expect(value.isEnabled)
        #expect(value.exitLocations.count == maxLimit)
        #expect(value.entryLocations.count == maxLimit)
    }

    @Test("Adds locations up to the default limit (50) for either entry or exit")
    func addDuplicate() throws {
        let repository = makeRepository()
        var recentConnections: RecentConnections?
        var thrownError: Error?

        repository
            .recentConnectionsPublisher
            .sink(receiveValue: { result in
                switch result {
                case .success(let value):
                    recentConnections = value
                case .failure(let error):
                    thrownError = error
                }
            })
            .store(in: &cancellables)

        repository.enable(se, selectedExitRelays: de)
        repository.add(
            UserSelectedRelays(
                locations: se.locations,
                customListSelection: UserSelectedRelays.CustomListSelection(listId: UUID(), isList: true)),
            selectedExitRelays: se)
        repository.add(
            UserSelectedRelays(
                locations: se.locations,
                customListSelection: UserSelectedRelays.CustomListSelection(listId: UUID(), isList: false)),
            selectedExitRelays: se)
        repository.add(de, selectedExitRelays: nl)

        let value = try #require(recentConnections)
        #expect(thrownError == nil)
        #expect(value.isEnabled)
        #expect(value.entryLocations.count == 3)
        #expect(value.exitLocations.count == 3)
    }

    @Test("Removes all recents connections with disabling recents")
    func disable() throws {
        let repository = makeRepository()

        var recentConnections: RecentConnections?
        var thrownError: Error?

        repository
            .recentConnectionsPublisher
            .sink(receiveValue: { result in
                switch result {
                case .success(let value):
                    recentConnections = value
                case .failure(let error):
                    thrownError = error
                }
            })
            .store(in: &cancellables)

        repository.disable()

        let value = try #require(recentConnections)
        #expect(thrownError == nil)
        #expect(!value.isEnabled)
        #expect(value.entryLocations.count == 0)
        #expect(value.exitLocations.count == 0)

    }

    @Test("Fails with an error if a location is added while recents are disabled.")
    func addRecentsBeforeEnablingRecents() throws {
        let repository = makeRepository()
        repository.disable()

        var recentConnections: RecentConnections?
        var thrownError: Error?
        repository
            .recentConnectionsPublisher
            .sink(receiveValue: { result in
                switch result {
                case .success(let value):
                    recentConnections = value
                case .failure(let error):
                    thrownError = error
                }
            })
            .store(in: &cancellables)
        repository.add(de, selectedExitRelays: se)

        let error = try #require(thrownError as? RecentConnectionsRepositoryError)
        #expect(error == RecentConnectionsRepositoryError.recentsDisabled)
        #expect(recentConnections == nil)
    }

    @Test("Removes list items and clears references when a custom list is deleted")
    func cleanupDeletedCustomList() throws {
        let repository = makeRepository()
        var recentConnections: RecentConnections?
        var thrownError: Error?

        repository
            .recentConnectionsPublisher
            .sink { result in
                switch result {
                case .success(let value):
                    recentConnections = value
                case .failure(let error):
                    thrownError = error
                }
            }
            .store(in: &cancellables)

        let deletedListId = UUID()

        let listItem = UserSelectedRelays(
            locations: se.locations + de.locations,
            customListSelection: .init(listId: deletedListId, isList: true)
        )

        let referencingItem = UserSelectedRelays(
            locations: de.locations,
            customListSelection: .init(listId: deletedListId, isList: false)
        )

        let unrelatedItem = UserSelectedRelays(locations: nl.locations)

        repository.enable(listItem, selectedExitRelays: se)
        repository.add(referencingItem, selectedExitRelays: de)
        repository.add(unrelatedItem, selectedExitRelays: nl)

        repository.deleteCustomList(deletedListId)

        let value = try #require(recentConnections)
        #expect(thrownError == nil)

        #expect(value.entryLocations.count == 2)
        #expect(value.entryLocations.contains { $0.locations == de.locations && $0.customListSelection == nil })
        #expect(value.entryLocations.contains { $0.locations == nl.locations })

        #expect(value.exitLocations.count == 3)
    }

    private func makeRepository(max: UInt = 50) -> RecentConnectionsRepository {
        return RecentConnectionsRepository(store: InMemorySettingsStore<SettingNotFound>(), maxLimit: max)
    }
}
