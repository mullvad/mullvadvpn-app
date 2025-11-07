//
//  CustomListInteractor.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-15.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes

protocol CustomListInteractorProtocol {
    func fetch(by id: UUID) -> CustomList?
    func fetchAll() -> [CustomList]
    func save(list: CustomList) throws
    func delete(customList: CustomList)
}

class CustomListInteractor: CustomListInteractorProtocol, @unchecked Sendable {

    private enum FinishAction {
        case save, delete
    }

    private let tunnelManager: TunnelManager
    private let repository: CustomListRepositoryProtocol

    init(
        tunnelManager: TunnelManager,
        repository: CustomListRepositoryProtocol
    ) {
        self.tunnelManager = tunnelManager
        self.repository = repository
    }

    func fetch(by id: UUID) -> CustomList? {
        repository.fetch(by: id)
    }

    func fetchAll() -> [CustomList] {
        repository.fetchAll()
    }

    func save(list: CustomList) throws {
        try repository.save(list: list)
        updateRelayConstraint(list: list, action: .save)
    }

    func delete(customList: CustomList) {
        repository.delete(id: customList.id)
        updateRelayConstraint(list: customList, action: .delete)
    }

    private func updateRelayConstraint(list: CustomList, action: FinishAction) {
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

        relayConstraints.entryLocations = self.updateRelayConstraint(
            relayConstraints.entryLocations,
            for: action,
            in: list
        )
        relayConstraints.exitLocations = self.updateRelayConstraint(
            relayConstraints.exitLocations,
            for: action,
            in: list
        )

        tunnelManager.updateSettings([.relayConstraints(relayConstraints)])
    }

    private func updateRelayConstraint(
        _ relayConstraint: RelayConstraint<UserSelectedRelays>,
        for action: FinishAction,
        in list: CustomList
    ) -> RelayConstraint<UserSelectedRelays> {
        var relayConstraint = relayConstraint

        guard let customListSelection = relayConstraint.value?.customListSelection,
            customListSelection.listId == list.id
        else { return relayConstraint }

        switch action {
        case .save:
            if customListSelection.isList {
                let selectedRelays = UserSelectedRelays(
                    locations: list.locations,
                    customListSelection: UserSelectedRelays.CustomListSelection(listId: list.id, isList: true)
                )
                relayConstraint = .only(selectedRelays)
            } else {
                let selectedConstraintIsRemovedFromList = list.locations.filter {
                    relayConstraint.value?.locations.contains($0) ?? false
                }.isEmpty

                if selectedConstraintIsRemovedFromList {
                    relayConstraint = .only(UserSelectedRelays(locations: []))
                }
            }
        case .delete:
            relayConstraint = .only(UserSelectedRelays(locations: []))
        }

        return relayConstraint
    }
}
