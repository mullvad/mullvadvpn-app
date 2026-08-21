// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
        lhs.node == rhs.node && lhs.node.children.count == rhs.node.children.count && lhs.section == rhs.section
            && lhs.isSelected == rhs.isSelected
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
