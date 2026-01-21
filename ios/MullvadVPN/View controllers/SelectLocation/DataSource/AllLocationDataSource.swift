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
    var cities: [String: CityRelays] // keyed by cityCode

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
    /// Only creates country nodes initially - children are created lazily when expanded.
    func reload(_ relays: LocationRelays) {
        let expandedCodes = collectExpandedCodes()

        // Group relays by country and city for lazy loading
        relaysByCountry = [:]
        var countryActiveState: [String: Bool] = [:]

        for relay in relays.relays {
            guard let serverLocation = relays.locations[relay.location.rawValue] else { continue }

            let countryCode = String(relay.location.country)
            let cityCode = String(relay.location.city)

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
            }
        }

        // Create only country nodes initially
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
            countryNode.hasLazyChildren = true
            countryNodes.append(countryNode)

            // If this country was expanded, populate its children immediately
            if expandedCodes.contains(countryCode) {
                populateChildren(for: countryNode, expandedCodes: expandedCodes)
            }
        }

        countryNodes.sort()
        nodes = countryNodes
    }

    /// Populates children for a node if they haven't been created yet.
    /// Call this when a node is about to be expanded.
    func populateChildren(for node: LocationNode, expandedCodes: Set<String> = []) {
        guard node.hasLazyChildren, node.children.isEmpty else { return }

        // Check if this is a country node
        if let countryData = relaysByCountry[node.code] {
            populateCountryChildren(node: node, countryData: countryData, expandedCodes: expandedCodes)
            return
        }

        // Check if this is a city node (code format: "countryCode-cityCode")
        let codeParts = node.code.split(separator: "-")
        if codeParts.count >= 2 {
            let countryCode = String(codeParts[0])
            let cityCode = String(codeParts[1])
            if let cityData = relaysByCountry[countryCode]?.cities[cityCode] {
                populateCityChildren(node: node, countryCode: countryCode, cityCode: cityCode, cityData: cityData)
            }
        }
    }

    private func populateCountryChildren(
        node: LocationNode,
        countryData: RelaysByLocation,
        expandedCodes: Set<String>
    ) {
        var cityNodes: [LocationNode] = []
        let countryCode = node.code

        for (cityCode, cityData) in countryData.cities {
            let countryCityCode = LocationNode.combineNodeCodes([countryCode, cityCode])
            let cityLocation = RelayLocation.city(countryCode, cityCode)

            // Check if city has any active relays
            let hasActiveRelay = cityData.relays.contains { $0.active }

            let cityNode = LocationNode(
                name: NSLocalizedString(cityData.cityName, comment: ""),
                code: countryCityCode,
                locations: [cityLocation],
                isActive: hasActiveRelay,
                parent: node,
                showsChildren: expandedCodes.contains(countryCityCode)
            )
            cityNode.hasLazyChildren = true
            cityNodes.append(cityNode)

            // If this city was expanded, populate its children immediately
            if expandedCodes.contains(countryCityCode) {
                populateCityChildren(node: cityNode, countryCode: countryCode, cityCode: cityCode, cityData: cityData)
            }
        }

        cityNodes.sort()
        node.children = cityNodes
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
        return descendantNode(in: rootNode, for: location, baseCodes: [])
    }
}
