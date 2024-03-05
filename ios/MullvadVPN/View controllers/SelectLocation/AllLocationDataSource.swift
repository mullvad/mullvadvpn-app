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
    func reload(_ response: REST.ServerRelaysResponse, relays: [REST.ServerRelay]) {
        let rootNode = RootLocationNode()

        for relay in relays {
            guard case
                let .city(countryCode, cityCode) = RelayLocation(dashSeparatedString: relay.location),
                let serverLocation = response.locations[relay.location]
            else { continue }

            let relayLocation = RelayLocation.hostname(countryCode, cityCode, relay.hostname)

            for ancestorOrSelf in relayLocation.ancestors + [relayLocation] {
                addLocation(ancestorOrSelf, rootNode: rootNode, serverLocation: serverLocation, relay: relay)
            }
        }

        nodes = rootNode.children
    }

    func node(by location: RelayLocation) -> LocationNode? {
        let rootNode = RootLocationNode(children: nodes)

        return switch location {
        case let .country(countryCode):
            rootNode.descendantNodeFor(code: countryCode)
        case let .city(_, cityCode):
            rootNode.descendantNodeFor(code: cityCode)
        case let .hostname(_, _, hostCode):
            rootNode.descendantNodeFor(code: hostCode)
        }
    }

    private func addLocation(
        _ location: RelayLocation,
        rootNode: LocationNode,
        serverLocation: REST.ServerLocation,
        relay: REST.ServerRelay
    ) {
        switch location {
        case let .country(countryCode):
            let countryNode = CountryLocationNode(
                name: serverLocation.country,
                code: countryCode,
                locations: [location]
            )

            if !rootNode.children.contains(countryNode) {
                rootNode.children.append(countryNode)
                rootNode.children.sort()
            }

        case let .city(countryCode, cityCode):
            let cityNode = CityLocationNode(name: serverLocation.city, code: cityCode, locations: [location])

            if let countryNode = rootNode.countryFor(code: countryCode),
               !countryNode.children.contains(cityNode) {
                cityNode.parent = countryNode
                countryNode.children.append(cityNode)
                countryNode.children.sort()
            }

        case let .hostname(countryCode, cityCode, hostCode):
            let hostNode = HostLocationNode(name: relay.hostname, code: hostCode, locations: [location])

            if let countryNode = rootNode.countryFor(code: countryCode),
               let cityNode = countryNode.cityFor(code: cityCode),
               !cityNode.children.contains(hostNode) {
                hostNode.parent = cityNode
                cityNode.children.append(hostNode)
                cityNode.children.sort()
            }
        }
    }
}
