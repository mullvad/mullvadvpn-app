//
//  RecentListDataSource.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-11-18.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
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
            recents.compactMap { userSelectedRelays -> RecentLocationNode? in
                let allLocationNode = allLocationDataSource.node(by: userSelectedRelays)
                guard
                    let node =
                        customListsDataSource.node(by: userSelectedRelays)
                        ?? allLocationNode
                else { return nil }

                // Preserve the parent only when the node originates from a custom list
                let copiedNode = node.copy(withParent: node.root.asCustomListNode)

                return RecentLocationNode(
                    name: copiedNode.name,
                    code: copiedNode.code,
                    locations: copiedNode.locations,
                    isActive: copiedNode.isActive,
                    parent: copiedNode.parent,
                    children: copiedNode.children,
                    showsChildren: false,  // Recents shouldn't be expandable
                    isHiddenFromSearch: true,  // Recents shouldn't be searchable
                    locationInfo: allLocationNode?.pathToRoot())  // Store relay location info (country, city)

            }
            .filter({ $0.isActive })
            .prefix(3)
        )

    }

    func node(by selectedRelays: UserSelectedRelays) -> LocationNode? {
        nodes.first { node in
            node.userSelectedRelays == selectedRelays
        }
    }
}
