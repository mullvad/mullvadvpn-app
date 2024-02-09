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
        if text.isEmpty {
            return locationList
        } else {
            var filteredLocations: [RelayLocation] = []
            locationList.forEach { location in
                guard let countryNode = nodeByLocation[location] else { return }
                countryNode.showsChildren = false

                if text.isEmpty || countryNode.displayName.fuzzyMatch(text) {
                    filteredLocations.append(countryNode.location)
                }

                for cityNode in countryNode.children {
                    cityNode.showsChildren = false

                    let relaysContainSearchString = cityNode.children
                        .contains(where: { $0.displayName.fuzzyMatch(text) })

                    if cityNode.displayName.fuzzyMatch(text) || relaysContainSearchString {
                        if !filteredLocations.contains(where: { $0 == countryNode.location }) {
                            filteredLocations.append(countryNode.location)
                        }

                        filteredLocations.append(cityNode.location)
                        countryNode.showsChildren = true

                        if relaysContainSearchString {
                            cityNode.children.map { $0.location }.forEach {
                                filteredLocations.append($0)
                            }
                            cityNode.showsChildren = true
                        }
                    }
                }
            }

            return filteredLocations
        }
    }

    func reload(
        _ response: MullvadREST.REST.ServerRelaysResponse,
        relays: [MullvadREST.REST.ServerRelay]
    ) -> [RelayLocation] {
        nodeByLocation.removeAll()
        let rootNode = self.makeRootNode(name: SelectLocationGroup.allLocations.description)

        for relay in relays {
            guard case let .city(countryCode, cityCode) = RelayLocation(dashSeparatedString: relay.location),
                  let serverLocation = response.locations[relay.location] else { continue }

            let relayLocation = RelayLocation.hostname(countryCode, cityCode, relay.hostname)

            for ascendantOrSelf in relayLocation.ascendants + [relayLocation] {
                guard !nodeByLocation.keys.contains(ascendantOrSelf) else {
                    continue
                }

                // Maintain the `showsChildren` state when transitioning between relay lists
                let wasShowingChildren = nodeByLocation[ascendantOrSelf]?.showsChildren ?? false

                let node = createNode(
                    root: rootNode,
                    ascendantOrSelf: ascendantOrSelf,
                    serverLocation: serverLocation,
                    relay: relay,
                    wasShowingChildren: wasShowingChildren
                )
                nodeByLocation[ascendantOrSelf] = node
            }
        }

        rootNode.sortChildrenRecursive()
        rootNode.computeActiveChildrenRecursive()
        locationList = rootNode.flatRelayLocationList()
        return locationList
    }
}
