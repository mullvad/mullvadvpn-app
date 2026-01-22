//
//  CustomListLocationNodeBuilder.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-03-14.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import MullvadTypes

struct CustomListLocationNodeBuilder {
    let customList: CustomList
    let allLocations: [LocationNode]
    /// Optional data source to populate lazy children when looking up hostnames
    let dataSource: AllLocationDataSource?

    init(customList: CustomList, allLocations: [LocationNode], dataSource: AllLocationDataSource? = nil) {
        self.customList = customList
        self.allLocations = allLocations
        self.dataSource = dataSource
    }

    var customListLocationNode: CustomListLocationNode {
        let listNode = CustomListLocationNode(
            name: customList.name,
            code: customList.name,
            locations: customList.locations,
            isActive: true,  // Defaults to true, updated after children have been populated.
            customList: customList
        )

        // Create root node once and reuse for all lookups
        let rootNode = RootLocationNode(children: allLocations)

        listNode.children = listNode.locations.compactMap { location in
            switch location {
            case let .country(countryCode):
                return
                    rootNode
                    .countryFor(code: countryCode)?
                    .copy(withParent: listNode)

            case let .city(countryCode, cityCode):
                return
                    rootNode
                    .countryFor(code: countryCode)?
                    .cityFor(codes: [countryCode, cityCode])?
                    .copy(withParent: listNode)

            case let .hostname(countryCode, cityCode, hostCode):
                // For hostname lookups, we need to ensure the city's relay children are populated
                if let cityNode =
                    rootNode
                    .countryFor(code: countryCode)?
                    .cityFor(codes: [countryCode, cityCode])
                {
                    // Populate lazy children if needed
                    dataSource?.populateChildren(for: cityNode)
                    return
                        cityNode
                        .hostFor(code: hostCode)?
                        .copy(withParent: listNode)
                }
                return nil
            }
        }

        listNode.isActive = !listNode.children.isEmpty
        listNode.sort()

        return listNode
    }
}

private extension CustomListLocationNode {
    func sort() {
        let sortedChildren = Dictionary(
            grouping: children,
            by: {
                return switch RelayLocation(dashSeparatedString: $0.code)! {
                case .country:
                    LocationGroup.country
                case .city:
                    LocationGroup.city
                case .hostname:
                    LocationGroup.host
                }
            }
        )
        .sorted(by: { $0.key < $1.key })
        .reduce([]) {
            $0 + $1.value.sorted(by: { $0.name < $1.name })
        }

        children = sortedChildren
    }
}

private enum LocationGroup: CaseIterable, Comparable {
    case country, city, host
}
