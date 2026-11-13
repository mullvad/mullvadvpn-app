// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadREST
import MullvadSettings
import MullvadTypes

@MainActor
protocol CustomListInteractorProtocol {
    func fetch(by id: UUID) -> CustomList?
    func fetchAll() -> [CustomList]
    func save(list: CustomList) throws
    func delete(customList: CustomList)
    func availableLocations(context: MultihopContext) -> [LocationNode]
    func addLocationToCustomList(relayLocations: [RelayLocation], customListName: String) throws
    func removeLocationFromCustomList(relayLocations: [RelayLocation], customListName: String) throws
}

struct CustomListInteractor: CustomListInteractorProtocol {

    private enum CustomListAction {
        case save, delete
    }

    private let tunnelManager: SettingsUpdating
    private let repository: CustomListRepositoryProtocol
    private let recentsRepository: RecentConnectionsRepositoryProtocol
    private let relaySelector: RelaySelectorProtocol

    init(
        tunnelManager: SettingsUpdating,
        repository: CustomListRepositoryProtocol,
        recentsRepository: RecentConnectionsRepositoryProtocol,
        relaySelector: RelaySelectorProtocol
    ) {
        self.tunnelManager = tunnelManager
        self.repository = repository
        self.recentsRepository = recentsRepository
        self.relaySelector = relaySelector
    }

    func fetch(by id: UUID) -> CustomList? {
        repository.fetch(by: id)
    }

    func fetchAll() -> [CustomList] {
        repository.fetchAll()
    }

    func save(list: CustomList) throws {
        try repository.save(list: list)
        updateCustomListRelayConstraints(list: list, action: .save)
    }

    func delete(customList: CustomList) {
        repository.delete(id: customList.id)
        recentsRepository.deleteCustomList(customList.id)
        updateCustomListRelayConstraints(list: customList, action: .delete)
    }

    func availableLocations(context: MultihopContext) -> [LocationNode] {
        var availableLocations = [LocationNode]()
        if let relaysCandidates = try? relaySelector.findCandidates(
            tunnelSettings: tunnelManager.settings.withAnyLocation,
            includeInactive: true
        ) {
            switch context {
            case .entry:
                if let entryRelays = relaysCandidates.entryRelays {
                    let dataSource = AllLocationDataSource()
                    dataSource.reload(entryRelays.toLocationRelays())
                    availableLocations = dataSource.nodes
                }
            case .exit:
                let dataSource = AllLocationDataSource()
                dataSource
                    .reload(relaysCandidates.exitRelays.toLocationRelays())
                availableLocations = dataSource.nodes
            }
        }
        return availableLocations
    }

    func addLocationToCustomList(relayLocations: [RelayLocation], customListName: String) throws {
        let customList =
            fetchAll().first { $0.name == customListName }
            ?? CustomList(
                name: customListName,
                locations: []
            )

        let allLocations = (customList.locations + relayLocations)
        let locations: [RelayLocation] =
            allLocations
            .filter { $0.ancestors.allSatisfy { !allLocations.contains($0) } }
            .reduce(
                [],
                { partialResult, location in
                    if !partialResult.contains(location) {
                        return partialResult + [location]
                    } else {
                        return partialResult
                    }
                })
        let newCustomList = CustomList(
            id: customList.id,
            name: customList.name,
            locations: locations
        )
        try save(list: newCustomList)
    }

    func removeLocationFromCustomList(
        relayLocations: [RelayLocation],
        customListName: String
    ) throws {
        let customList = fetchAll().first { $0.name == customListName }
        guard let customList else {
            return
        }
        let allLocations = customList.locations.filter {
            !relayLocations.contains($0)
        }
        let newCustomList = CustomList(
            id: customList.id,
            name: customList.name,
            locations: allLocations
        )
        try save(list: newCustomList)
    }

    private func updateCustomListRelayConstraints(list: CustomList, action: CustomListAction) {
        var relayConstraints = tunnelManager.settings.relayConstraints

        // only update relay constraints if custom list is currently selected
        var isSelectionAffected = false
        if let customListExitSelection = relayConstraints.exitLocations.value?.customListSelection {
            if customListExitSelection.listId == list.id {
                isSelectionAffected = true
            }
        }
        if let customListEntrySelection = relayConstraints.entryLocations.value?.customListSelection {
            if customListEntrySelection.listId == list.id {
                isSelectionAffected = true
            }
        }
        guard isSelectionAffected else {
            return
        }

        let newEntryLocations = self.updateRelayConstraint(
            relayConstraints.entryLocations,
            for: action,
            in: list
        )

        let newExitLocations = self.updateRelayConstraint(
            relayConstraints.exitLocations,
            for: action,
            in: list
        )

        if newExitLocations.value != relayConstraints.exitLocations.value
            || newEntryLocations.value != relayConstraints.entryLocations.value
        {
            relayConstraints.exitLocations = newExitLocations
            relayConstraints.entryLocations = newEntryLocations
            tunnelManager
                .updateSettings(
                    [.relayConstraints(relayConstraints)],
                    completionHandler: nil
                )
        }
    }

    private func updateRelayConstraint(
        _ relayConstraint: RelayConstraint<UserSelectedRelays>,
        for action: CustomListAction,
        in list: CustomList
    ) -> RelayConstraint<UserSelectedRelays> {
        var relayConstraint = relayConstraint

        guard let customListSelection = relayConstraint.value?.customListSelection,
            customListSelection.listId == list.id
        else {
            // leave constraint untouched if custom list is not selected
            return relayConstraint
        }

        switch action {
        case .save:
            // update constraint to custom list
            if customListSelection.isList {
                // the selection is the list itself. In that case, update the constraint
                let selectedRelays = UserSelectedRelays(
                    locations: list.locations,
                    customListSelection: UserSelectedRelays.CustomListSelection(listId: list.id, isList: true)
                )
                relayConstraint = .only(selectedRelays)
            } else {
                // the selection is a location inside a custom list
                let selectedConstraintIsRemovedFromList = list.locations.allSatisfy { listLocation in
                    !(relayConstraint.value?.locations
                        .flatMap { [$0] + $0.ancestors }
                        .contains(listLocation) ?? false)
                }

                if selectedConstraintIsRemovedFromList {
                    // remove location from constraint if it is removed from the custom list
                    // this will lead to the blocked state
                    relayConstraint = .only(UserSelectedRelays(locations: relayConstraint.value?.locations ?? []))
                }
            }
        case .delete:
            // update constraint to custom list
            if customListSelection.isList {
                // remove list from constraint
                // this will lead to the blocked state
                relayConstraint = .only(UserSelectedRelays(locations: []))
            } else {
                // fallback to all locations
                relayConstraint = .only(UserSelectedRelays(locations: relayConstraint.value?.locations ?? []))
            }
        }

        return relayConstraint
    }
}

protocol SettingsUpdating {
    func updateSettings(_ updates: [TunnelSettingsUpdate], completionHandler: (@Sendable () -> Void)?)
    var settings: LatestTunnelSettings { get }
}

class SettingsUpdatingMock: SettingsUpdating {
    var updateCalled = false
    func updateSettings(
        _ updates: [MullvadSettings.TunnelSettingsUpdate],
        completionHandler: (@Sendable () -> Void)?
    ) {
        for update in updates {
            update.apply(to: &settings)
        }
        updateCalled = true
    }

    var settings = LatestTunnelSettings()
}

struct MockCustomListInteractor: CustomListInteractorProtocol {
    func availableLocations(context: MultihopContext) -> [LocationNode] {
        []
    }

    func fetch(by id: UUID) -> MullvadSettings.CustomList? {
        nil
    }

    func fetchAll() -> [MullvadSettings.CustomList] {
        []
    }

    func save(list: MullvadSettings.CustomList) throws {}

    func delete(customList: MullvadSettings.CustomList) {}

    func addLocationToCustomList(relayLocations: [MullvadTypes.RelayLocation], customListName: String) throws {}

    func removeLocationFromCustomList(relayLocations: [MullvadTypes.RelayLocation], customListName: String) throws {}
}

extension TunnelManager: SettingsUpdating {}
