//
//  AllLocationDataSource.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-22.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes

/// Relay data grouped for lazy child node creation
struct RelaysByLocation {
    var countryName: String
    var cities: [String: CityRelays]  // keyed by cityCode

    struct CityRelays {
        var cityName: String
        var relays: [REST.ServerRelay]
    }
}

class AllLocationDataSource: SearchableLocationDataSource {
    private(set) var nodes = [LocationNode]()

    /// Stored relay data for lazy child creation, keyed by countryCode
    private var relaysByCountry: [String: RelaysByLocation] = [:]

    /// Constructs a collection of node trees from relays fetched from the API.
    /// Creates country and city nodes eagerly, but relay/host nodes are created lazily when expanded.
    /// This hybrid approach maintains performance (relays are ~40,000) while allowing custom lists
    /// and selection to work correctly (they need to traverse countries and cities).
    func reload(_ relays: LocationRelays) {
        let expandedCodes = collectExpandedCodes()

        // Group relays by country and city for lazy loading of hosts
        relaysByCountry = [:]
        var countryActiveState: [String: Bool] = [:]
        var cityActiveState: [String: Bool] = [:]

        for relay in relays.relays {
            guard let serverLocation = relays.locations[relay.location.rawValue] else { continue }

            let countryCode = String(relay.location.country)
            let cityCode = String(relay.location.city)
            let countryCityCode = LocationNode.combineNodeCodes([countryCode, cityCode])

            // Initialize country entry if needed
            if relaysByCountry[countryCode] == nil {
                relaysByCountry[countryCode] = RelaysByLocation(
                    countryName: serverLocation.country,
                    cities: [:]
                )
            }

            // Initialize city entry if needed
            if relaysByCountry[countryCode]?.cities[cityCode] == nil {
                relaysByCountry[countryCode]?.cities[cityCode] = RelaysByLocation.CityRelays(
                    cityName: serverLocation.city,
                    relays: []
                )
            }

            relaysByCountry[countryCode]?.cities[cityCode]?.relays.append(relay)

            // Track active state
            if relay.active {
                countryActiveState[countryCode] = true
                cityActiveState[countryCityCode] = true
            }
        }

        // Create country and city nodes eagerly, but relay nodes lazily
        var countryNodes: [LocationNode] = []
        for (countryCode, countryData) in relaysByCountry {
            let countryLocation = RelayLocation.country(countryCode)
            let countryNode = LocationNode(
                name: NSLocalizedString(countryData.countryName, comment: ""),
                code: countryCode,
                locations: [countryLocation],
                isActive: countryActiveState[countryCode] ?? false,
                showsChildren: expandedCodes.contains(countryCode)
            )

            // Create city nodes eagerly (needed for custom lists and selection)
            var cityNodes: [LocationNode] = []
            for (cityCode, cityData) in countryData.cities {
                let countryCityCode = LocationNode.combineNodeCodes([countryCode, cityCode])
                let cityLocation = RelayLocation.city(countryCode, cityCode)

                let cityNode = LocationNode(
                    name: NSLocalizedString(cityData.cityName, comment: ""),
                    code: countryCityCode,
                    locations: [cityLocation],
                    isActive: cityActiveState[countryCityCode] ?? false,
                    parent: countryNode,
                    showsChildren: expandedCodes.contains(countryCityCode)
                )
                // Mark city as having lazy children (relays)
                cityNode.hasLazyChildren = true
                cityNodes.append(cityNode)

                // If this city was expanded, populate its relay children immediately
                if expandedCodes.contains(countryCityCode) {
                    populateCityChildren(
                        node: cityNode, countryCode: countryCode, cityCode: cityCode, cityData: cityData)
                }
            }

            cityNodes.sort()
            countryNode.children = cityNodes
            countryNodes.append(countryNode)
        }

        countryNodes.sort()
        nodes = countryNodes
    }

    /// Populates relay children for a city node if they haven't been created yet.
    /// Call this when a city node is about to be expanded.
    func populateChildren(for node: LocationNode, expandedCodes: Set<String> = []) {
        guard node.hasLazyChildren, node.children.isEmpty else { return }

        // City node code format: "countryCode-cityCode"
        let codeParts = node.code.split(separator: "-")
        if codeParts.count >= 2 {
            let countryCode = String(codeParts[0])
            let cityCode = String(codeParts[1])
            if let cityData = relaysByCountry[countryCode]?.cities[cityCode] {
                populateCityChildren(node: node, countryCode: countryCode, cityCode: cityCode, cityData: cityData)
            }
        }
    }

    private func populateCityChildren(
        node: LocationNode,
        countryCode: String,
        cityCode: String,
        cityData: RelaysByLocation.CityRelays
    ) {
        var hostNodes: [LocationNode] = []

        for relay in cityData.relays {
            let hostLocation = RelayLocation.hostname(countryCode, cityCode, relay.hostname)
            let hostNode = LocationNode(
                name: relay.hostname,
                code: relay.hostname,
                locations: [hostLocation],
                isActive: relay.active,
                parent: node
            )
            hostNodes.append(hostNode)
        }

        hostNodes.sort()
        node.children = hostNodes
    }

    /// Efficiently collects codes of all nodes that have showsChildren = true.
    private func collectExpandedCodes() -> Set<String> {
        var codes = Set<String>()
        func collect(_ node: LocationNode) {
            if node.showsChildren {
                codes.insert(node.code)
            }
            for child in node.children {
                collect(child)
            }
        }
        for node in nodes {
            collect(node)
        }
        return codes
    }

    func node(by selectedRelays: UserSelectedRelays) -> LocationNode? {
        let rootNode = RootLocationNode(children: nodes)
        guard let location = selectedRelays.locations.first else {
            return nil
        }

        // For hostname lookups, ensure the city's relay children are populated first
        if case let .hostname(countryCode, cityCode, _) = location {
            if let cityNode =
                rootNode
                .countryFor(code: countryCode)?
                .cityFor(codes: [countryCode, cityCode])
            {
                populateChildren(for: cityNode)
            }
        }

        return descendantNode(in: rootNode, for: location, baseCodes: [])
    }
}
