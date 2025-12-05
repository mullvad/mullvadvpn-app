//
//  CustomListsDataSource.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-22.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadSettings
import MullvadTypes

class CustomListsDataSource: LocationDataSourceProtocol {
    private(set) var nodes = [LocationNode]()
    private(set) var repository: CustomListRepositoryProtocol

    init(repository: CustomListRepositoryProtocol) {
        self.repository = repository
    }

    /// Constructs a collection of node trees by copying each matching counterpart
    /// from the complete list of nodes created in ``AllLocationDataSource``.
    func reload(allLocationNodes: [LocationNode]) {
        let expandedRelays = nodes.flatMap { [$0] + $0.flattened }.filter { $0.showsChildren }.map { $0.code }
        nodes = repository.fetchAll().map { list in
            let customListWrapper = CustomListLocationNodeBuilder(customList: list, allLocations: allLocationNodes)
            let listNode = customListWrapper.customListLocationNode
            listNode.showsChildren = expandedRelays.contains(listNode.code)

            listNode.forEachDescendant { node in
                // Each item in a section in a diffable data source needs to be unique.
                // Since LocationCellViewModel partly depends on LocationNode.code for
                // equality, each node code needs to be prefixed with the code of its
                // parent custom list to uphold this.
                node.code = LocationNode.combineNodeCodes([listNode.code, node.code])
                node.showsChildren = expandedRelays.contains(node.code)
            }

            return listNode
        }
    }

    func node(by selectedRelays: UserSelectedRelays) -> LocationNode? {
        let rootNode = RootLocationNode(children: nodes)
        guard
            let selection = selectedRelays.customListSelection,
            let selectedNode = rootNode.children.first(where: {
                $0.asCustomListNode?.customList.id == selection.listId
            })
        else { return nil }

        if selection.isList {
            return selectedNode
        }

        if let location = selectedRelays.locations.first {
            return descendantNode(
                in: rootNode,
                for: location,
                baseCodes: [selectedNode.code]
            )
        }

        return nil
    }

}
