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
                let node =
                    if userSelectedRelays.customListSelection != nil {
                        customListsDataSource.node(by: userSelectedRelays)
                    } else {
                        allLocationDataSource.node(by: userSelectedRelays)
                    }

                guard let copiedNode = node?.copy() else { return nil }
                copiedNode.isHiddenFromSearch = true
                copiedNode.showsChildren = false
                return copiedNode
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
