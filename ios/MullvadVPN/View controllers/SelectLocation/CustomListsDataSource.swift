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

    func reload(allLocationNodes: [LocationNode]) {
        nodes = repository.fetchAll().map { list in
            let listNode = ListLocationNode(
                nodeName: list.name,
                nodeCode: list.name.lowercased(),
                locations: list.locations,
                customList: list
            )

            listNode.children = list.locations.compactMap { location in
                copy(location, from: allLocationNodes, withParent: listNode)
            }

            listNode.forEachDescendant { node in
                node.nodeCode = "\(listNode.nodeCode)-\(node.nodeCode)"
            }

            return listNode
        }
    }

    func node(by locations: [RelayLocation], for customList: CustomList) -> LocationNode? {
        guard let customListNode = nodes.first(where: { $0.nodeName == customList.name })
        else { return nil }

        if locations.count > 1 {
            return customListNode
        } else {
            return switch locations.first {
            case let .country(countryCode):
                customListNode.childNodeFor(nodeCode: "\(customListNode.nodeCode)-\(countryCode)")
            case let .city(_, cityCode):
                customListNode.childNodeFor(nodeCode: "\(customListNode.nodeCode)-\(cityCode)")
            case let .hostname(_, _, hostCode):
                customListNode.childNodeFor(nodeCode: "\(customListNode.nodeCode)-\(hostCode)")
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
                .countryFor(countryCode: countryCode)?.copy(withParent: parentNode)

        case let .city(countryCode, cityCode):
            rootNode
                .countryFor(countryCode: countryCode)?.copy(withParent: parentNode)
                .cityFor(cityCode: cityCode)

        case let .hostname(countryCode, cityCode, hostCode):
            rootNode
                .countryFor(countryCode: countryCode)?.copy(withParent: parentNode)
                .cityFor(cityCode: cityCode)?
                .hostFor(hostCode: hostCode)
        }
    }
}
