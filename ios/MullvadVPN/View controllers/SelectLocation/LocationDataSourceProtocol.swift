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
            node.children.forEach { $0.isHiddenFromSearch = false }
        }
        node.isHiddenFromSearch = !matchesSelf && !childMatches
        node.showsChildren = childMatches
        return node.isHiddenFromSearch
    }

    private func reset() {
        func resetChildren(node: LocationNode) {
            node.children.forEach { child in
                child.isHiddenFromSearch = false
                child.showsChildren = false
                resetChildren(node: child)
            }
        }
        nodes.forEach { node in
            node.isHiddenFromSearch = false
            node.showsChildren = false
            resetChildren(node: node)
        }
    }

    func search(by text: String) -> [LocationNode] {
        guard !text.isEmpty else {
            reset()
            return nodes
        }
        nodes.forEach { node in
            _ = hideInSearch(
                node: node,
                searchText: text
            )
        }

        return nodes
    }
}
