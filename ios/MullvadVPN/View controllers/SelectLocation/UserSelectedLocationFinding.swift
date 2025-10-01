//
//  UserSelectedLocationFinding.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-10-01.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import MullvadTypes

protocol UserSelectedLocationFinder {
    func node(_ selectedRelays: UserSelectedRelays) -> LocationNode?
}

struct UserSelectedLocationFinding: UserSelectedLocationFinder {
    let allLocationsDataSource: AllLocationDataSource
    let customListsDataSource: CustomListsDataSource

    private func customListNode(_ selectedRelays: UserSelectedRelays) -> LocationNode? {
        // Look for a matching custom list node.
        if let customListSelection = selectedRelays.customListSelection,
           let customList = customListsDataSource.customList(by: customListSelection.listId),
           let selectedNode = customListsDataSource.node(by: selectedRelays, for: customList) {
            return selectedNode
        }
        return nil
    }

    private func relayNode(_ selectedRelays: UserSelectedRelays) -> LocationNode? {
        // Look for a matching node.
        if let location = selectedRelays.locations.first,
           let selectedNode = allLocationsDataSource.node(by: location) {
            return selectedNode
        }
        return nil
    }

    func node(_ selectedRelays: UserSelectedRelays) -> LocationNode? {
        customListNode(selectedRelays) ?? relayNode(selectedRelays)
    }
}
