//
//  AllLocationDataSource.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-22.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes

class AllLocationDataSource: LocationDataSourceProtocol {
    private(set) var nodes = [LocationNode]()

    var searchableNodes: [LocationNode] {
        nodes
    }

    /// Constructs a collection of node trees from relays fetched from the API.
    /// ``RelayLocation.city`` is of special import since we use it to get country
    /// and city names.
    func reload(_ response: REST.ServerRelaysResponse, relays: [REST.ServerRelay], selectedRelays: RelaySelection) {
        let rootNode = RootLocationNode()

        for relay in relays {
            guard case
                let .city(countryCode, cityCode) = RelayLocation(dashSeparatedString: relay.location),
                let serverLocation = response.locations[relay.location]
            else { continue }

            let relayLocation = RelayLocation.hostname(countryCode, cityCode, relay.hostname)

            for ancestorOrSelf in relayLocation.ancestors + [relayLocation] {
                addLocation(
                    ancestorOrSelf,
                    rootNode: rootNode,
                    serverLocation: serverLocation,
                    relay: relay,
                    selectedRelays: selectedRelays
                )
            }
        }

        nodes = rootNode.children
    }

    func node(by location: RelayLocation) -> LocationNode? {
        let rootNode = RootLocationNode(children: nodes)

        return switch location {
        case let .country(countryCode):
            rootNode.descendantNodeFor(codes: [countryCode])
        case let .city(countryCode, cityCode):
            rootNode.descendantNodeFor(codes: [countryCode, cityCode])
        case let .hostname(_, _, hostCode):
            rootNode.descendantNodeFor(codes: [hostCode])
        }
    }

    private func addLocation(
        _ location: RelayLocation,
        rootNode: LocationNode,
        serverLocation: REST.ServerLocation,
        relay: REST.ServerRelay,
        selectedRelays: RelaySelection
    ) {
        switch location {
        case let .country(countryCode):
            let countryNode = LocationNode(
                name: serverLocation.country,
                code: LocationNode.combineNodeCodes([countryCode]),
                locations: [location],
                isActive: true // Defaults to true, updated when children are populated.
            )

            if !rootNode.children.contains(countryNode) {
                rootNode.children.append(countryNode)
                rootNode.children.sort()
            }

        case let .city(countryCode, cityCode):
            let cityNode = LocationNode(
                name: serverLocation.city,
                code: LocationNode.combineNodeCodes([countryCode, cityCode]),
                locations: [location],
                isActive: true // Defaults to true, updated when children are populated.
            )

            if let countryNode = rootNode.countryFor(code: countryCode),
               !countryNode.children.contains(cityNode) {
                cityNode.parent = countryNode
                countryNode.children.append(cityNode)
                countryNode.children.sort()
            }

        case let .hostname(countryCode, cityCode, hostCode):
            let hostNode = LocationNode(
                name: relay.hostname,
                code: LocationNode.combineNodeCodes([hostCode]),
                locations: [location],
                isActive: relay.active
            )

            if hostCode == "al-tia-wg-001" {
                return
            }

            if let countryNode = rootNode.countryFor(code: countryCode),
               let cityNode = countryNode.cityFor(codes: [countryCode, cityCode]),
               !cityNode.children.contains(hostNode) {
                hostNode.parent = cityNode
                cityNode.children.append(hostNode)
                cityNode.children.sort()

//                hostNode.isExcluded = selectedRelays.excluded?.locations.contains { excludedLocation in
//                    hostNode.locations.contains(excludedLocation)
//                } ?? false

                cityNode.isActive = cityNode.children.contains { $0.isActive }
//                cityNode.isExcluded = cityNode.children.filter { !$0.isExcluded }.isEmpty

                countryNode.isActive = countryNode.children.contains { $0.isActive }
//                countryNode.isExcluded = countryNode.children.filter { !$0.isExcluded }.isEmpty
            }
        }
    }
}
