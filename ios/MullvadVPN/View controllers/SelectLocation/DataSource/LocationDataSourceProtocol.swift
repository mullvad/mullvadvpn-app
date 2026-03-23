//
//  LocationDataSourceProtocol.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-02-07.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
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
    func search(by text: String) -> [LocationNode] {
        guard !text.isEmpty else {
            return nodes
        }
        let results =
            nodes
            .compactMap { searchTree($0, searchText: text) }
            .flatMap(flattenResults)
            .sorted {
                if $0.score == $1.score {
                    return $0.node.name < $1.node.name
                }
                return $0.score > $1.score
            }
            .map { $0.node }
        return results
    }

    private func searchTree(
        _ node: LocationNode,
        searchText: String
    ) -> NodeResult? {

        let selfScore = node.name.search(searchText)
        let selfMatches = selfScore != .none

        var childResults: [NodeResult] = []

        for child in node.children {
            if let result = searchTree(child, searchText: searchText) {
                childResults.append(result)
            }
        }

        if !selfMatches && childResults.isEmpty {
            return nil
        }

        let bestChildScore = childResults.map(\.score).max() ?? .none
        let bestScore = max(selfScore, bestChildScore)

        return NodeResult(
            node: node,
            score: bestScore,
            matchedSelf: selfMatches,
            matchedChildren: childResults
        )
    }

    private func flattenResults(_ result: NodeResult) -> [NodeResult] {
        let children = result.matchedChildren

        // Collapse if parent itself matched
        if result.matchedSelf {
            return [result]
        }

        // If only one child AND parent doesn't match then skip parent
        if result.node.children.count == 1 && !result.matchedSelf {
            return flattenResults(children.first!)
        }

        // Collapse if ALL children matched
        if !children.isEmpty && children.count == result.node.children.count {
            return [result]
        }

        // Repopulate node
        if !children.isEmpty {
            return children.flatMap(flattenResults)
        }

        return []
    }
}

extension LocationDataSourceProtocol {
    func setConnectedRelay(hostname: String?) {
        nodes.forEachNode { node in
            node.isConnected = node.name == hostname
        }
    }

    /// Excludeds nodes from being selectable. A node gets excluded if the selection only allows for one possible relay.
    /// This is used in multihop to make sure that the during relay selection entry and exit can different.
    /// It prevent the user from making a selection that would lead to the blocked state.
    /// - Parameters:
    ///   - excludedSelection: The selection that should be checked for exclusion.
    func setExcludedNode(excludedSelection: UserSelectedRelays?) {
        nodes.forEachNode { node in
            node.isExcluded = false
        }
        guard let selectedRelayLocations = excludedSelection?.locations,
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

    /// Efficiently collects codes of all nodes that have showsChildren = true.
    func collectExpandedCodes() -> Set<String> {
        var codes = Set<String>()

        for node in self.nodes {
            if node.showsChildren {
                codes.insert(node.code)
                node.forEachDescendant { child in
                    if child.showsChildren {
                        codes.insert(child.code)
                    }
                }
            }
        }
        return codes
    }
}
private struct NodeResult {
    let node: LocationNode
    let score: SearchScore
    let matchedSelf: Bool
    let matchedChildren: [NodeResult]
}
