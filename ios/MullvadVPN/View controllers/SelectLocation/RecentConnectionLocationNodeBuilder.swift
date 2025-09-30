//
//  RecentConnectionLocationNodeBuilder.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-09-30.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import Foundation
import MullvadSettings
import MullvadTypes

struct RecentConnectionLocationNodeBuilder {
    let recentConnection: RecentConnection
    let allLocations: [LocationNode]
    let settings: LatestTunnelSettings
    
    init(recentConnection: RecentConnection, allLocations: [LocationNode], settings: LatestTunnelSettings) {
        self.recentConnection = recentConnection
        self.allLocations = allLocations
        self.settings = settings
    }
    
    var recentConnectionLocationNode: RecentConnectionLocationNode {
        let name = settings.tunnelMultihopState.isEnabled ? self.recentConnection.entry?.customListSelection.name
        let recentConnectionLocationNode = RecentConnectionLocationNode(
//        let rece = CustomListLocationNode(
//            name: customList.name,
//            code: customList.name,
//            locations: customList.locations,
//            isActive: true, // Defaults to true, updated after children have been populated.
//            customList: customList
//        )
//
//        listNode.children = listNode.locations.compactMap { location in
//            let rootNode = RootLocationNode(children: allLocations)
//
//            return switch location {
//            case let .country(countryCode):
//                rootNode
//                    .countryFor(code: countryCode)?
//                    .copy(withParent: listNode)
//
//            case let .city(countryCode, cityCode):
//                rootNode
//                    .countryFor(code: countryCode)?
//                    .cityFor(codes: [countryCode, cityCode])?
//                    .copy(withParent: listNode)
//
//            case let .hostname(countryCode, cityCode, hostCode):
//                rootNode
//                    .countryFor(code: countryCode)?
//                    .cityFor(codes: [countryCode, cityCode])?
//                    .hostFor(code: hostCode)?
//                    .copy(withParent: listNode)
//            }
//        }
//
//        listNode.isActive = !listNode.children.isEmpty
//        listNode.sort()
//
//        return listNode
    }
    
}
