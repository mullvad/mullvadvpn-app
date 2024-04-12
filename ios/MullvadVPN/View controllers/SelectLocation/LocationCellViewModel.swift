//
//  LocationCellViewModel.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-02-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

struct LocationCellViewModel: Hashable {
    let section: LocationSection
    let node: LocationNode
    var indentationLevel = 0
    var isSelected = false

    func hash(into hasher: inout Hasher) {
        hasher.combine(node)
        hasher.combine(node.children.count)
        hasher.combine(section)
        hasher.combine(isSelected)
        hasher.combine(indentationLevel)
    }

    static func == (lhs: Self, rhs: Self) -> Bool {
        lhs.node == rhs.node &&
            lhs.node.children.count == rhs.node.children.count &&
            lhs.section == rhs.section &&
            lhs.isSelected == rhs.isSelected &&
            lhs.indentationLevel == rhs.indentationLevel
    }
}

extension [LocationCellViewModel] {
    mutating func addSubNodes(from item: LocationCellViewModel, at indexPath: IndexPath) {
        let section = LocationSection.allCases[indexPath.section]
        let row = indexPath.row + 1

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
