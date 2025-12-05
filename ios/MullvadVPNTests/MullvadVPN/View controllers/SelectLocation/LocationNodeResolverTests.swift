//
//  LocationNodeResolverTests.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-12-04.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadMockData
import Testing

@testable import MullvadSettings
@testable import MullvadTypes

struct LocationNodeResolverTests {
    let customListDataSource: CustomListsDataSource
    let allLocationDataSource: AllLocationDataSource
    let recentListDataSource: RecentListDataSource
    let resolver: LocationNodeResolver

    init() {
        let response = ServerRelaysResponseStubs.sampleRelays
        let relays = LocationRelays(relays: response.wireguard.relays, locations: response.locations)

        allLocationDataSource = AllLocationDataSource()
        customListDataSource = CustomListsDataSource(
            repository: CustomListsRepositoryStub(customLists: Self.customLists))

        allLocationDataSource.reload(relays)
        customListDataSource.reload(allLocationNodes: allLocationDataSource.nodes)

        recentListDataSource = RecentListDataSource(allLocationDataSource, customListsDataSource: customListDataSource)
        recentListDataSource.reload(Self.recents)

        resolver = LocationNodeResolver(providers: [recentListDataSource, customListDataSource, allLocationDataSource])
    }

    @Test
    func testSingleSelection() {
        resolver.setSelectedNode(selectedRelays: Self.recents.first!)
        let allNodes = allLocationDataSource.nodes + customListDataSource.nodes + recentListDataSource.nodes
        let count = allNodes.count(where: { $0.flattened.contains(where: { $0.isSelected }) })
        #expect(allNodes.count(where: { $0.flattened.contains(where: { $0.isSelected }) }) == 1)
    }
}

extension LocationNodeResolverTests {
    private static var customLists: [CustomList] {
        [
            CustomList(
                name: "Netflix",
                locations: [
                    .hostname("es", "mad", "es1-wireguard"),
                    .country("se"),
                    .city("us", "dal"),
                ]),
            CustomList(
                name: "Youtube",
                locations: [
                    .hostname("se", "sto", "se2-wireguard"),
                    .city("us", "dal"),
                ]),
        ]
    }

    private static var recents: [UserSelectedRelays] {
        [
            UserSelectedRelays(locations: [.country("se")]),
            UserSelectedRelays(
                locations: customLists.first!.locations,
                customListSelection: UserSelectedRelays.CustomListSelection(listId: customLists.first!.id, isList: true)
            ),
        ]
    }
}
