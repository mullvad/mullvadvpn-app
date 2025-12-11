//
//  LocationNodeResolver.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-11-20.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

final class LocationNodeResolver {
    private let providers: [LocationDataSourceProtocol]

    init(providers: [LocationDataSourceProtocol]) {
        self.providers = providers
    }

    func setSelectedNodeExpanded(_ isExpanded: Bool) {
        let selectedNode = first { provider in
            let rootNode = RootLocationNode(children: provider.nodes)
            return rootNode
                .flattened
                .first { $0.isSelected }
        }
        selectedNode?.forEachAncestor { $0.showsChildren = isExpanded }
    }

    func setSelectedNode(selectedRelays: UserSelectedRelays) {
        resetSelection()
        let selectedNode = first(where: { provider in
            provider.node(by: selectedRelays)
        })
        selectedNode?.isSelected = true
    }

    func setConnectedRelay(hostname: String?) {
        // Skip the "Recent" section when building the root node used for the connected server indicator
        let rootNode = RootLocationNode(children: providers[1...].flatMap(\.nodes))
        rootNode
            .flattened
            .forEachNode { node in
                node.isConnected = node.name == hostname
            }
    }

    /// Excluded nodes from being selectable. A node gets excluded if the selection only allows for one possible relay.
    /// This is used in multihop to make sure that the during relay selection entry and exit can different.
    /// It prevent the user from making a selection that would lead to the blocked state.
    /// - Parameters:
    ///   - excludedSelection: The selection that should be checked for exclusion.
    func setExcludedNode(excludedSelection: UserSelectedRelays) {
        guard excludedSelection.locations.count == 1 else { return }
        resetExclusion()
        let allNodes = providers.flatMap(\.nodes)
        allNodes.forEachNode { node in
            let locations = Set((node.flattened + [node]).flatMap { $0.locations })
            if locations.contains(excludedSelection.locations) && node.activeRelayNodes.count == 1 {
                node.isExcluded = true
                node.forEachDescendant { child in
                    child.isExcluded = true
                }
            }
        }

    }
    private func first(where predicate: (LocationDataSourceProtocol) -> LocationNode?) -> LocationNode? {
        for provider in providers {
            if let node = predicate(provider) {
                return node
            }
        }
        return nil
    }

    private func resetSelection() {
        providers.forEach { provider in
            provider.nodes.forEachNode { node in
                node.isSelected = false
                node.showsChildren = false
            }
        }
    }

    private func resetExclusion() {
        providers.forEach { provider in
            provider.nodes.forEachNode { node in
                node.isExcluded = false
            }
        }
    }
}
