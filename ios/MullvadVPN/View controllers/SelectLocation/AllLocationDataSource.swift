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
    func reload(_ relays: LocationRelays) {
        let rootNode = RootLocationNode()
        let expandedRelays = nodes.flatMap { [$0] + $0.flattened }.filter { $0.showsChildren }.map { $0.code }

        for relay in relays.relays {
            guard
                let serverLocation = relays.locations[relay.location.rawValue]
            else { continue }

            let relayLocation = RelayLocation.hostname(
                String(relay.location.country),
                String(relay.location.city),
                relay.hostname
            )

            for ancestorOrSelf in relayLocation.ancestors + [relayLocation] {
                addLocation(
                    ancestorOrSelf,
                    rootNode: rootNode,
                    serverLocation: serverLocation,
                    relay: relay,
                    showsChildren: expandedRelays.contains(ancestorOrSelf.stringRepresentation)
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
        showsChildren: Bool
    ) {
        switch location {
        case let .country(countryCode):
            let countryNode = LocationNode(
                name: serverLocation.country,
                code: LocationNode.combineNodeCodes([countryCode]),
                locations: [location],
                isActive: true, // Defaults to true, updated when children are populated.
                showsChildren: showsChildren
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
                isActive: true, // Defaults to true, updated when children are populated.
                showsChildren: showsChildren
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
                isActive: relay.active,
                showsChildren: showsChildren
            )

            if let countryNode = rootNode.countryFor(code: countryCode),
               let cityNode = countryNode.cityFor(codes: [countryCode, cityCode]),
               !cityNode.children.contains(hostNode) {
                hostNode.parent = cityNode
                cityNode.children.append(hostNode)
                cityNode.children.sort()

                cityNode.isActive = cityNode.children.contains(where: { hostNode in
                    hostNode.isActive
                })

                countryNode.isActive = countryNode.children.contains(where: { cityNode in
                    cityNode.isActive
                })
            }
        }
    }
}
