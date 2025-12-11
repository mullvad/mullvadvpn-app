//
//  LocationDataSourceProtocol.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-02-07.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes

protocol SearchableLocationDataSource: LocationDataSourceProtocol {}

protocol LocationDataSourceProtocol {
    var nodes: [LocationNode] { get }
    func node(by selectedRelays: UserSelectedRelays) -> LocationNode?
}
extension SearchableLocationDataSource {
    func search(by text: String) {
        nodes.forEachNode { node in
            node.isHiddenFromSearch = false
            node.showsChildren = false
        }
        guard !text.isEmpty else {
            return
        }
        nodes.forEach { node in
            _ = hideInSearch(
                node: node,
                searchText: text
            )
        }
    }

    private func hideInSearch(node: LocationNode, searchText: String) -> Bool {
        let matchesSelf = node.name.fuzzyMatch(searchText)
        var childMatches = false
        for child in node.children where !hideInSearch(node: child, searchText: searchText) {
            childMatches = true
        }
        if matchesSelf && !childMatches {
            node.forEachDescendant { child in
                child.isHiddenFromSearch = false
                child.showsChildren = false
            }
        }
        node.isHiddenFromSearch = !matchesSelf && !childMatches
        node.showsChildren = childMatches
        return node.isHiddenFromSearch
    }
}
extension LocationDataSourceProtocol {
    func descendantNode(
        in rootNode: LocationNode,
        for location: RelayLocation,
        baseCodes: [String]
    ) -> LocationNode? {
        let descendantNodeFor: ([String]) -> LocationNode? = { codes in
            return switch location {
            case let .country(countryCode):
                rootNode.descendantNodeFor(codes: codes + [countryCode])
            case let .city(countryCode, cityCode):
                rootNode.descendantNodeFor(codes: codes + [countryCode, cityCode])
            case let .hostname(_, _, hostCode):
                rootNode.descendantNodeFor(codes: codes + [hostCode])
            }
        }
        return descendantNodeFor(baseCodes)
    }
}
