//
//  AllLocationDataSource.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-02-07.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes

class AllLocationDataSource: LocationDataSourceProtocol {
    var nodeByLocation = [RelayLocation: SelectLocationNode]()
    private var locationList = [RelayLocation]()

    func search(by text: String) -> [RelayLocation] {
        guard !text.isEmpty else {
            return locationList
        }

        var filteredLocations: [RelayLocation] = []
        locationList.forEach { location in
            guard let countryNode = nodeByLocation[location] else { return }
            countryNode.showsChildren = false

            if countryNode.displayName.fuzzyMatch(text) {
                filteredLocations.append(countryNode.location)
            }

            countryNode.children.forEach { cityNode in
                cityNode.showsChildren = false

                let relaysContainSearchString = cityNode.children
                    .contains(where: { $0.displayName.fuzzyMatch(text) })

                if cityNode.displayName.fuzzyMatch(text) || relaysContainSearchString {
                    if !filteredLocations.contains(countryNode.location) {
                        filteredLocations.append(countryNode.location)
                    }

                    filteredLocations.append(cityNode.location)
                    countryNode.showsChildren = true

                    if relaysContainSearchString {
                        filteredLocations.append(contentsOf: cityNode.children.map { $0.location })
                        cityNode.showsChildren = true
                    }
                }
            }
        }

        return filteredLocations
    }

    func reload(
        _ response: REST.ServerRelaysResponse,
        relays: [REST.ServerRelay]
    ) -> [RelayLocation] {
        nodeByLocation.removeAll()
        let rootNode = self.makeRootNode(name: SelectLocationSection.allLocations.description)

        for relay in relays {
            guard case let .city(countryCode, cityCode) = RelayLocation(dashSeparatedString: relay.location),
                  let serverLocation = response.locations[relay.location] else { continue }

            let relayLocation = RelayLocation.hostname(countryCode, cityCode, relay.hostname)

            for ancestorOrSelf in relayLocation.ancestors + [relayLocation] {
                guard !nodeByLocation.keys.contains(ancestorOrSelf) else {
                    continue
                }

                // Maintain the `showsChildren` state when transitioning between relay lists
                let wasShowingChildren = nodeByLocation[ancestorOrSelf]?.showsChildren ?? false

                let node = createNode(
                    root: rootNode,
                    ancestorOrSelf: ancestorOrSelf,
                    serverLocation: serverLocation,
                    relay: relay,
                    wasShowingChildren: wasShowingChildren
                )
                nodeByLocation[ancestorOrSelf] = node
            }
        }

        rootNode.sortChildrenRecursive()
        rootNode.computeActiveChildrenRecursive()
        locationList = rootNode.flatRelayLocationList()
        return locationList
    }
}
