//
//  RecentListDataSource.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-11-18.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadSettings
import MullvadTypes

class RecentListDataSource: LocationDataSourceProtocol {
    private(set) var nodes = [LocationNode]()
    let allLocationDataSource: AllLocationDataSource
    let customListsDataSource: CustomListsDataSource

    init(_ allLocationDataSource: AllLocationDataSource, customListsDataSource: CustomListsDataSource) {
        self.allLocationDataSource = allLocationDataSource
        self.customListsDataSource = customListsDataSource
    }

    func reload(_ recents: [UserSelectedRelays]) {
        // Build the `nodes` array from the user's recently selected locations.
        // For each entry in `recents`, resolve it to a `LocationNode` using the custom
        // list data source first, falling back to the all location list. Keep up to 3 results.
        self.nodes = Array(
            recents.map({ (userSelectedRelays) -> LocationNode? in
                customListsDataSource.node(by: userSelectedRelays)
                    ?? allLocationDataSource.node(by: userSelectedRelays)
            })
            .compactMap({ $0?.copy() })
            .prefix(3))
    }

    func node(by selectedRelays: UserSelectedRelays) -> LocationNode? {
        return self.nodes.first(where: {
            if let customListSelection = selectedRelays.customListSelection,
                customListSelection.isList
            {
                return $0.locations == selectedRelays.locations
            }
            return $0.locations == selectedRelays.locations
        })
    }
}
