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

    var customListLocationNode: CustomListLocationNode {
        let listNode = CustomListLocationNode(
            name: customList.name,
            code: customList.name,
            locations: customList.locations,
            isActive: true,  // Defaults to true, updated after children have been populated.
            customList: customList
        )

        listNode.children = listNode.locations.compactMap { location in
            let rootNode = RootLocationNode(children: allLocations)

            return switch location {
            case let .country(countryCode):
                rootNode
                    .countryFor(code: countryCode)?
                    .copy(withParent: listNode)

            case let .city(countryCode, cityCode):
                rootNode
                    .countryFor(code: countryCode)?
                    .cityFor(codes: [countryCode, cityCode])?
                    .copy(withParent: listNode)

            case let .hostname(countryCode, cityCode, hostCode):
                rootNode
                    .countryFor(code: countryCode)?
                    .cityFor(codes: [countryCode, cityCode])?
                    .hostFor(code: hostCode)?
                    .copy(withParent: listNode)
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
