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

protocol LocationDataSourceProtocol {
    var nodes: [LocationNode] { get }
}

extension LocationDataSourceProtocol {
    private func match(name: String, searchText: String) -> Bool {
        name.lowercased().hasPrefix(searchText.lowercased())
    }

    private func hideInSearch(node: LocationNode, searchText: String) -> Bool {
        let matchesSelf = match(name: node.name, searchText: searchText)
        var childMatches = false
        for child in node.children where !hideInSearch(node: child, searchText: searchText) {
            childMatches = true
        }
        if matchesSelf && !childMatches {
            resetChildren(node: node)
        }
        node.isHiddenFromSearch = !matchesSelf && !childMatches
        node.showsChildren = childMatches
        return node.isHiddenFromSearch
    }

    private func resetChildren(node: LocationNode) {
        node.children.forEach { child in
            child.isHiddenFromSearch = false
            child.showsChildren = false
            resetChildren(node: child)
        }
    }

    private func reset() {
        nodes.forEach { node in
            node.isHiddenFromSearch = false
            node.showsChildren = false
            resetChildren(node: node)
        }
    }

    func search(by text: String) {
        guard !text.isEmpty else {
            reset()
            return
        }
        nodes.forEach { node in
            _ = hideInSearch(
                node: node,
                searchText: text
            )
        }
    }
}
