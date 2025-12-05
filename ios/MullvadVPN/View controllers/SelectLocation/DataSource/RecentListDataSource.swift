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
        // Resolve each recent selection to a location node by checking custom lists first,
        // then falling back to the main location data source. Convert each resolved node
        // and keep only the first three results.
        self.nodes = Array(
            recents.map({ (userSelectedRelays) -> LocationNode? in
                customListsDataSource.node(by: userSelectedRelays)
                    ?? allLocationDataSource.node(by: userSelectedRelays)
            })
            .compactMap({ $0?.copy() })
            .prefix(3))
    }

    func node(by selectedRelays: UserSelectedRelays) -> LocationNode? {
        self.nodes.first(where: {
            let userSelectedRelays = $0.userSelectedRelays
            if let customListSelection = userSelectedRelays.customListSelection,
                customListSelection.isList
            {
                return userSelectedRelays.locations == $0.locations
            }
            return userSelectedRelays.locations == $0.locations
        })
    }
}
