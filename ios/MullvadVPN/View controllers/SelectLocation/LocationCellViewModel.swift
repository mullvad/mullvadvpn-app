//
//  LocationCellViewModel.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-02-05.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

struct LocationCellViewModel: Hashable, Sendable {
    let section: LocationSection
    let node: LocationNode
    var indentationLevel = 0
    var isSelected = false
    var excludedRelayTitle: String?

    func hash(into hasher: inout Hasher) {
        hasher.combine(node)
        hasher.combine(node.children.count)
        hasher.combine(section)
        hasher.combine(isSelected)
    }

    static func == (lhs: Self, rhs: Self) -> Bool {
        lhs.node == rhs.node &&
            lhs.node.children.count == rhs.node.children.count &&
            lhs.section == rhs.section &&
            lhs.isSelected == rhs.isSelected
    }
}

extension [LocationCellViewModel] {
    mutating func addSubNodes(from item: LocationCellViewModel, at indexPath: IndexPath) {
        let section = LocationSection.allCases[indexPath.section]
        let row = indexPath.row + 1
        item.node.showsChildren = true

        let locations = item.node.children.map {
            LocationCellViewModel(
                section: section,
                node: $0,
                indentationLevel: item.indentationLevel + 1,
                isSelected: item.isSelected
            )
        }

        if row < count {
            insert(contentsOf: locations, at: row)
        } else {
            append(contentsOf: locations)
        }
    }

    mutating func removeSubNodes(from node: LocationNode) {
        for node in node.children {
            node.showsChildren = false
            removeAll(where: { node == $0.node })

            removeSubNodes(from: node)
        }
    }
}

extension LocationCellViewModel {
    /* Exclusion of other locations in the same node tree (as the currently excluded location)
     happens when there are no more hosts in that tree that can be selected.
     We check this by doing the following, in order:

     1. Count hostnames in the tree. More than one means that there are other locations than
     the excluded one for the relay selector to choose from. No exlusion.

     2. Count hostnames in the excluded node. More than one means that there are multiple
     locations for the relay selector to choose from. No exclusion.

     3. Check existance of a location in the tree that matches the currently excluded location.
     No match means no exclusion.
     */
    func shouldExcludeLocation(_ excludedLocation: LocationCellViewModel?) -> Bool {
        guard let excludedLocation else {
            return false
        }

        let proxyNode = RootLocationNode(children: [node])
        let allLocations = Set(proxyNode.flattened.flatMap { $0.locations })
        let hostCount = allLocations.filter { location in
            if case .hostname = location { true } else { false }
        }.count

        // If the there's more than one selectable relay in the current node we don't need
        // to show this in the location tree and can return early.
        guard hostCount == 1 else { return false }

        let proxyExcludedNode = RootLocationNode(children: [excludedLocation.node])
        let allExcludedLocations = Set(proxyExcludedNode.flattened.flatMap { $0.locations })
        let excludedHostCount = allExcludedLocations.filter { location in
            if case .hostname = location { true } else { false }
        }.count

        // If the there's more than one selectable relay in the excluded node we don't need
        // to show this in the location tree and can return early.
        guard excludedHostCount == 1 else { return false }

        var containsExcludedLocation = false
        if allLocations.contains(where: { allExcludedLocations.contains($0) }) {
            containsExcludedLocation = true
        }

        // If the tree doesn't contain the excluded node we do nothing, otherwise the
        // required conditions have now all been met.
        return containsExcludedLocation
    }
}
