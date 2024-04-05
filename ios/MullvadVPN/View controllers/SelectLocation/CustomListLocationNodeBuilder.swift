//
//  CustomListLocationNodeBuilder.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-03-14.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

struct CustomListLocationNodeBuilder {
    let customList: CustomList
    let allLocations: [LocationNode]

    var customListLocationNode: CustomListLocationNode {
        let listNode = CustomListLocationNode(
            name: customList.name,
            code: customList.name.lowercased(),
            locations: customList.locations,
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
        return listNode
    }
}
