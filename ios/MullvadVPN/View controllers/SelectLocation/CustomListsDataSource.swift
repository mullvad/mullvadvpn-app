//
//  CustomListsDataSource.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-22.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
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

    var searchableNodes: [LocationNode] {
        nodes.flatMap { $0.children }
    }

    /// Constructs a collection of node trees by copying each matching counterpart
    /// from the complete list of nodes created in ``AllLocationDataSource``.
    func reload(allLocationNodes: [LocationNode], isFiltered: Bool) {
        // nodes.removeAll()

        nodes = repository.fetchAll().compactMap { customList in
            let listNode = CustomListLocationNode(
                name: customList.name,
                code: customList.name.lowercased(),
                locations: customList.locations,
                customList: customList
            )

            listNode.children = customList.locations.compactMap { location in
                copy(location, from: allLocationNodes, withParent: listNode)
            }

            listNode.forEachDescendant { node in
                // Each item in a section in a diffable data source needs to be unique.
                // Since LocationCellViewModel partly depends on LocationNode.code for
                // equality, each node code needs to be prefixed with the code of its
                // parent custom list to uphold this.
                node.code = LocationNode.combineNodeCodes([listNode.code, node.code])
            }

            return (isFiltered && listNode.children.isEmpty) ? nil : listNode
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

    private func copy(
        _ location: RelayLocation,
        from allLocationNodes: [LocationNode],
        withParent parentNode: LocationNode
    ) -> LocationNode? {
        let rootNode = RootLocationNode(children: allLocationNodes)

        return switch location {
        case let .country(countryCode):
            rootNode
                .countryFor(code: countryCode)?
                .copy(withParent: parentNode)

        case let .city(countryCode, cityCode):
            rootNode
                .countryFor(code: countryCode)?
                .cityFor(codes: [countryCode, cityCode])?
                .copy(withParent: parentNode)

        case let .hostname(countryCode, cityCode, hostCode):
            rootNode
                .countryFor(code: countryCode)?
                .cityFor(codes: [countryCode, cityCode])?
                .hostFor(code: hostCode)?
                .copy(withParent: parentNode)
        }
    }
}
