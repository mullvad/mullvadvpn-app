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

    func node(by relays: UserSelectedRelays, for customList: CustomList) -> LocationNode? {
        guard let listNode = nodes.first(where: { $0.name == customList.name }) else { return nil }

        if relays.customListSelection?.isList == true {
            return listNode
        } else {
            // Each search for descendant nodes needs the parent custom list node code to be
            // prefixed in order to get a match. See comment in reload() above.
            return switch relays.locations.first {
            case let .country(countryCode):
                listNode.descendantNodeFor(codes: [listNode.code, countryCode])
            case let .city(countryCode, cityCode):
                listNode.descendantNodeFor(codes: [listNode.code, countryCode, cityCode])
            case let .hostname(_, _, hostCode):
                listNode.descendantNodeFor(codes: [listNode.code, hostCode])
            case .none:
                nil
            }
        }
    }

    func customList(by id: UUID) -> CustomList? {
        repository.fetch(by: id)
    }
}
