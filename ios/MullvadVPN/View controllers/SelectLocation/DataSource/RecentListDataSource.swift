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
        nodes = Array(
            recents.compactMap { userSelectedRelays in
                // Resolve the original node once to preserve its hierarchy.
                let originalNode = allLocationDataSource.node(by: userSelectedRelays)

                // Copy from the custom list when the list itself is selected;
                // Otherwise fall back to copying from all locations.
                let copiedNode: LocationNode? = {
                    if let customList = userSelectedRelays.customListSelection,
                        customList.isList == true
                    {
                        return customListsDataSource.node(by: userSelectedRelays)?.copy()
                    } else {
                        return originalNode?.copy(withParent: originalNode)
                    }
                }()

                guard let node = copiedNode else { return nil }

                node.code = LocationNode.combineNodeCodes([UUID().uuidString, node.code])
                node.isHiddenFromSearch = true
                node.showsChildren = false
                return node
            }
            .prefix(3)
        )

    }

    func node(by selectedRelays: UserSelectedRelays) -> LocationNode? {
        nodes.first { node in
            node.locations == selectedRelays.locations
        }
    }
}
