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

    func setConnectedRelay(hostname: String?) {
        nodes.forEachNode { node in
            node.isConnected = node.name == hostname
        }
    }

    func setExcludedNode(excludedRelays: UserSelectedRelays?) {
        nodes.forEachNode { node in
            node.isExcluded = false
        }
        guard let selectedRelayLocations = excludedRelays?.locations,
            selectedRelayLocations.count == 1,
            let selectedRelayLocation = selectedRelayLocations.first
        else {
            return
        }
        nodes.forEachNode { node in
            let locations = Set((node.flattened + [node]).flatMap { $0.locations })
            if locations
                .contains(selectedRelayLocation) && node.activeRelayNodes.count == 1
            {
                node.isExcluded = true
                node.forEachDescendant { child in
                    child.isExcluded = true
                }
            }
        }
    }

    func setSelectedNode(selectedRelays: UserSelectedRelays?) {
        nodes.forEachNode { node in
            node.isSelected = false
        }
        guard let selectedRelays else { return }
        let selectedNode = node(by: selectedRelays)
        selectedNode?.isSelected = true
    }

    func expandSelection() {
        nodes.forEachNode { node in
            if node.isSelected {
                node.forEachAncestor { $0.showsChildren = true }
            }
        }
    }

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

    func node(by selectedRelays: UserSelectedRelays) -> LocationNode? {
        let rootNode = RootLocationNode(children: nodes)

        guard let location = selectedRelays.locations.first else {
            return nil
        }
        if let customListSelection = selectedRelays.customListSelection {
            let selectedCustomListNode = nodes.first(where: {
                ($0 as? CustomListLocationNode)?.customList.id == customListSelection.listId
            })

            guard let selectedCustomListNode else { return nil }

            if customListSelection.isList {
                return selectedCustomListNode
            }

            return switch location {
            case let .country(countryCode):
                rootNode.descendantNodeFor(codes: [selectedCustomListNode.code, countryCode])
            case let .city(countryCode, cityCode):
                rootNode.descendantNodeFor(codes: [selectedCustomListNode.code, countryCode, cityCode])
            case let .hostname(_, _, hostCode):
                rootNode.descendantNodeFor(codes: [selectedCustomListNode.code, hostCode])
            }

        } else {
            return switch location {
            case let .country(countryCode):
                rootNode.descendantNodeFor(codes: [countryCode])
            case let .city(countryCode, cityCode):
                rootNode.descendantNodeFor(codes: [countryCode, cityCode])
            case let .hostname(_, _, hostCode):
                rootNode.descendantNodeFor(codes: [hostCode])
            }
        }
    }
}
