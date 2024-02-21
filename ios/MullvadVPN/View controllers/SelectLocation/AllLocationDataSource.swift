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

    var searchableNodes: [LocationNode] { nodes }

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
            rootNode.childNodeFor(nodeCode: countryCode)
        case let .city(_, cityCode):
            rootNode.childNodeFor(nodeCode: cityCode)
        case let .hostname(_, _, hostCode):
            rootNode.childNodeFor(nodeCode: hostCode)
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
                nodeName: serverLocation.country,
                nodeCode: countryCode,
                locations: [location]
            )

            if !rootNode.children.contains(countryNode) {
                rootNode.children.append(countryNode)
                rootNode.children.sort()
            }

        case let .city(countryCode, cityCode):
            let cityNode = CityLocationNode(nodeName: serverLocation.city, nodeCode: cityCode, locations: [location])

            if let countryNode = rootNode.countryFor(countryCode: countryCode),
               !countryNode.children.contains(cityNode) {
                cityNode.parent = countryNode
                countryNode.children.append(cityNode)
                countryNode.children.sort()
            }

        case let .hostname(countryCode, cityCode, hostIdentifier):
            let hostNode = HostLocationNode(nodeName: relay.hostname, nodeCode: hostIdentifier, locations: [location])

            if let countryNode = rootNode.countryFor(countryCode: countryCode),
               let cityNode = countryNode.cityFor(cityCode: cityCode),
               !cityNode.children.contains(hostNode) {
                hostNode.parent = cityNode
                cityNode.children.append(hostNode)
                cityNode.children.sort()
            }
        }
    }
}
