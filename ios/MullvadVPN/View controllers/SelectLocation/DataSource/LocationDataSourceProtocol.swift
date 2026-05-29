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
    var selectedNode: LocationNode? { get set }
    func node(by selectedConstraint: RelayConstraint<UserSelectedRelays>) -> LocationNode?
}

extension SearchableLocationDataSource {
    func search(by text: String) -> [LocationNode] {
        guard !text.isEmpty else {
            return nodes
        }
        resetExpandedState()
        let results =
            nodes
            .compactMap { searchTree($0, searchText: text) }
            .flatMap(flattenResults)
            .sorted {
                if $0.score != $1.score {
                    return $0.score > $1.score
                }

                if $0.bestMatchIsSelf != $1.bestMatchIsSelf {
                    return $0.bestMatchIsSelf
                }

                return $0.node.name < $1.node.name
            }
            .map { $0.node }
        return results
    }

    private func searchTree(
        _ node: LocationNode,
        searchText: String
    ) -> NodeResult? {
        guard node.isSearchable else {
            return nil
        }

        let isCustomListNode = node.asCustomListNode != nil
        let name = isCustomListNode ? node.name : node.name.split(separator: "-").prefix(2).joined(separator: "-")
        let selfScore = name.search(searchText)
        let selfMatches = selfScore != .none

        let childResults: [NodeResult] = node.children.compactMap({
            searchTree($0, searchText: searchText)
        })

        if !selfMatches && childResults.isEmpty {
            return nil
        }

        let bestChildScore = childResults.map(\.score).max() ?? .none
        let bestScore = max(selfScore, bestChildScore)
        let bestMatchIsSelf = selfScore >= bestChildScore

        return NodeResult(
            node: node,
            score: bestScore,
            matchedSelf: selfMatches,
            bestMatchIsSelf: bestMatchIsSelf,
            matchedChildren: childResults
        )
    }

    private func flattenResults(_ result: NodeResult) -> [NodeResult] {
        let children = result.matchedChildren
        let totalChildren = result.node.children.count

        // Show the parent only if it matches AND it is the best match in its subtree
        if result.matchedSelf, result.bestMatchIsSelf {
            return [result]
        }

        if !result.bestMatchIsSelf {
            let matchedChildren = children.filter({ $0.score == result.score })
            // Collapse if ALL children matched
            if matchedChildren.count > 1 && matchedChildren.count == totalChildren {
                return [result]
            }
            return matchedChildren.flatMap(flattenResults)
        }

        // Repopulate node
        if !children.isEmpty {
            return children.flatMap(flattenResults)
        }

        return []
    }

    private func resetExpandedState() {
        nodes.forEachNode { node in
            node.forEachDescendant { $0.showsChildren = false }
        }
    }
}

extension LocationDataSourceProtocol {
    func setConnectedRelay(
        relayConstraint: RelayConstraint<UserSelectedRelays>,
        selectedRelay: SelectedRelay?
    ) {
        guard let selectedRelay else {
            return
        }

        let hostname =
            if relayConstraint == .any {
                AutomaticLocationNode().name
            } else {
                selectedRelay.hostname
            }

        nodes.forEachNode { node in
            node.isConnected = false

            if node.name == hostname {
                node.isConnected = true

                if let node = node.asAutomaticLocationNode {
                    node.locationInfo = [selectedRelay.location.country, selectedRelay.location.city]
                }
            }
        }
    }

    /// Excludes nodes from being selectable. A node gets excluded if the selection only allows for one possible relay.
    /// This is used in multihop to make sure that during relay selection entry and exit can be different.
    /// It prevent the user from making a selection that would lead to the blocked state.
    /// - Parameters:
    ///   - hostname: The selection that should be checked for exclusion.
    func setExcludedNode(hostname: String?) {
        nodes.forEachNode { node in
            node.isExcluded = false

            guard node.isActive else {
                return
            }

            let rootNode =
                if node is CustomListLocationNode {
                    // Since custom list nodes contain a copy of the node tree, remove self to avoid
                    // duplicates in the result.
                    RootLocationNode(children: node.children)
                } else {
                    node
                }

            let nodeHosts = rootNode.flattened.filter {
                return if case .hostname = $0.locations.first, $0.isActive {
                    true
                } else {
                    false
                }
            }

            node.isExcluded = (nodeHosts.count == 1) && (nodeHosts.first?.name == hostname)
        }
    }

    func setSelectedNode(constraint: RelayConstraint<UserSelectedRelays>) -> LocationNode? {
        // Although multiple equivalent nodes can be selected at the same time, we only need
        // the reference of one. Thus, the last one to be selected in the loop below will suffice.
        var selectedNode: LocationNode?

        nodes.forEachNode { node in
            // To determine if locations match.
            let nodeLocations = node.userSelectedRelays.locations
            let constraintLocations = constraint.value?.locations

            // To determine if custom lists match.
            let nodeCustomListSelection = node.userSelectedRelays.customListSelection
            let constraintCustomListSelection = constraint.value?.customListSelection

            let locationsMatch = nodeLocations == constraintLocations
            node.isSelected =
                if constraintLocations?.count == 0 {
                    // Empty constraint locations can happen when a custom list is removed.
                    false
                } else if constraint == .any {
                    // If using an automatic location we should check for a corresponding node.
                    node is AutomaticLocationNode
                } else if let nodeCustomListSelection {
                    // If the selection is a custom list, or in one, we should check for matching locations there.
                    locationsMatch && (nodeCustomListSelection == constraintCustomListSelection)
                } else {
                    // Otherwise we simply check for generally matching locations not part of any custom list.
                    locationsMatch && (constraintCustomListSelection == nil)
                }

            if node.isSelected {
                node.forEachAncestor { $0.showsChildren = true }
                selectedNode = node
            }
        }

        return selectedNode
    }

    /// Efficiently collects codes of all nodes that have showsChildren = true.
    func collectExpandedCodes() -> Set<String> {
        var codes = Set<String>()

        for node in nodes where node.showsChildren {
            codes.insert(node.code)

            node.forEachDescendant { child in
                if child.showsChildren {
                    codes.insert(child.code)
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
    let bestMatchIsSelf: Bool
    let matchedChildren: [NodeResult]
}
