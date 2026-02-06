//
//  AllLocationDataSource.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-22.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes

class AllLocationDataSource: SearchableLocationDataSource {
    private(set) var nodes = [LocationNode]()

    /// Constructs a collection of node trees from relays fetched from the API.
    /// ``RelayLocation.city`` is of special import since we use it to get country
    /// and city names.
    func reload(_ relays: LocationRelays) {
        let rootNode = RootLocationNode()
        let expandedCodes = collectExpandedCodes()

        // Use dictionaries for O(1) lookups during tree construction
        var countryNodesByCode: [String: LocationNode] = [:]
        var cityNodesByCode: [String: LocationNode] = [:]

        for relay in relays.relays {
            guard let serverLocation = relays.locations[relay.location.rawValue] else { continue }

            let countryCode = String(relay.location.country)
            let cityCode = String(relay.location.city)
            let countryCityCode = LocationNode.combineNodeCodes([countryCode, cityCode])

            // Get or create country node
            let countryNode: LocationNode
            if let existingCountry = countryNodesByCode[countryCode] {
                countryNode = existingCountry
            } else {
                let countryLocation = RelayLocation.country(countryCode)
                countryNode = LocationNode(
                    name: NSLocalizedString(serverLocation.country, comment: ""),
                    code: countryCode,
                    locations: [countryLocation],
                    isActive: true,
                    showsChildren: expandedCodes.contains(countryCode)
                )
                countryNodesByCode[countryCode] = countryNode
                rootNode.children.append(countryNode)
            }

            // Get or create city node
            let cityNode: LocationNode
            if let existingCity = cityNodesByCode[countryCityCode] {
                cityNode = existingCity
            } else {
                let cityLocation = RelayLocation.city(countryCode, cityCode)
                cityNode = LocationNode(
                    name: NSLocalizedString(serverLocation.city, comment: ""),
                    code: countryCityCode,
                    locations: [cityLocation],
                    isActive: true,
                    parent: countryNode,
                    showsChildren: expandedCodes.contains(countryCityCode)
                )
                cityNodesByCode[countryCityCode] = cityNode
                countryNode.children.append(cityNode)
            }

            // Create host node
            let hostLocation = RelayLocation.hostname(countryCode, cityCode, relay.hostname)
            let hostNode = LocationNode(
                name: relay.hostname,
                code: relay.hostname,
                locations: [hostLocation],
                isActive: relay.active,
                parent: cityNode,
                showsChildren: expandedCodes.contains(relay.hostname)
            )
            cityNode.children.append(hostNode)

            // Update active states
            if relay.active {
                cityNode.isActive = true
                countryNode.isActive = true
            }
        }

        // Update isActive for cities and countries that have no active relays
        for countryNode in rootNode.children {
            var countryHasActiveCity = false
            for cityNode in countryNode.children {
                let cityHasActiveHost = cityNode.children.contains { $0.isActive }
                cityNode.isActive = cityHasActiveHost
                if cityHasActiveHost {
                    countryHasActiveCity = true
                    continue
                }
            }
            countryNode.isActive = countryHasActiveCity
        }

        // Single sort pass at the end
        rootNode.children.sort()
        for countryNode in rootNode.children {
            countryNode.children.sort()
            for cityNode in countryNode.children {
                cityNode.children.sort()
            }
        }

        nodes = rootNode.children
    }

    func node(by selectedRelays: UserSelectedRelays) -> LocationNode? {
        let rootNode = RootLocationNode(children: nodes)
        guard let location = selectedRelays.locations.first else {
            return nil
        }
        return descendantNode(in: rootNode, for: location, baseCodes: [])
    }
}
