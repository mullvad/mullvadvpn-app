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
    private var selectedNode: LocationNode?

    init(providers: [LocationDataSourceProtocol]) {
        self.providers = providers
    }

    func setSelectedNodeExpanded(_ isExpanded: Bool) {
        selectedNode?.forEachAncestor { $0.showsChildren = isExpanded }
    }

    func setSelectedNode(selectedRelays: UserSelectedRelays) {
        resetSelection()
        selectedNode = node(by: selectedRelays)
        selectedNode?.isSelected = true
    }

    func setConnectedRelay(hostname: String?) {
        selectedNode?.flattened.forEachNode { node in
            node.isConnected = node.name == hostname
        }
    }

    /// Excluded nodes from being selectable. A node gets excluded if the selection only allows for one possible relay.
    /// This is used in multihop to make sure that the during relay selection entry and exit can different.
    /// It prevent the user from making a selection that would lead to the blocked state.
    /// - Parameters:
    ///   - excludedSelection: The selection that should be checked for exclusion.
    func setExcludedNode(excludedSelection: UserSelectedRelays?) {
        guard let excludedSelection,
            let excludedNode = node(by: excludedSelection)
        else { return }
        resetExclusion()
        excludedNode.isExcluded = excludedNode.activeRelayNodes.count == 1
    }

    private func node(by relays: UserSelectedRelays) -> LocationNode? {
        for provider in providers {
            if let node = provider.node(by: relays) {
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
