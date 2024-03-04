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
    func reload(allLocationNodes: [LocationNode]) {
        nodes = repository.fetchAll().map { list in
            let listNode = CustomListLocationNode(
                name: list.name,
                code: list.name.lowercased(),
                locations: list.locations,
                customList: list
            )

            listNode.children = list.locations.compactMap { location in
                copy(location, from: allLocationNodes, withParent: listNode)
            }

            listNode.forEachDescendant { node in
                // Each item in a section in a diffable data source needs to be unique.
                // Since LocationCellViewModel partly depends on LocationNode.code for
                // equality, each node code needs to be prefixed with the code of its
                // parent custom list to uphold this.
                node.code = "\(listNode.code)-\(node.code)"
            }

            return listNode
        }
    }

    func node(by locations: [RelayLocation], for customList: CustomList) -> LocationNode? {
        guard let customListNode = nodes.first(where: { $0.name == customList.name })
        else { return nil }

        if locations.count > 1 {
            return customListNode
        } else {
            // Each search for descendant nodes needs the parent custom list node code to be
            // prefixed in order to get a match. See comment in reload() above.
            return switch locations.first {
            case let .country(countryCode):
                customListNode.descendantNodeFor(code: "\(customListNode.code)-\(countryCode)")
            case let .city(_, cityCode):
                customListNode.descendantNodeFor(code: "\(customListNode.code)-\(cityCode)")
            case let .hostname(_, _, hostCode):
                customListNode.descendantNodeFor(code: "\(customListNode.code)-\(hostCode)")
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
                .countryFor(code: countryCode)?.copy(withParent: parentNode)

        case let .city(countryCode, cityCode):
            rootNode
                .countryFor(code: countryCode)?.copy(withParent: parentNode)
                .cityFor(code: cityCode)

        case let .hostname(countryCode, cityCode, hostCode):
            rootNode
                .countryFor(code: countryCode)?.copy(withParent: parentNode)
                .cityFor(code: cityCode)?
                .hostFor(code: hostCode)
        }
    }
}
