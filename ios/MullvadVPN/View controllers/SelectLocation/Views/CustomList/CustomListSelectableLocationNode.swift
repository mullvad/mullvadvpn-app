// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation

@Observable
class CustomListSelectableLocationNode: LocationNode, @unchecked Sendable {
    init(fromLocationNode locationNode: LocationNode) {
        super.init(
            name: locationNode.name,
            code: locationNode.code,
            locations: locationNode.locations,
            isActive: locationNode.isActive,
            parent: nil,
            children:
                locationNode.children
                .map {
                    CustomListSelectableLocationNode(
                        fromLocationNode: $0
                    )
                },
            showsChildren: locationNode.showsChildren,
        )
        children.forEach { $0.parent = self }
    }

    var customListAvailableChildren: [CustomListSelectableLocationNode] {
        children.compactMap { $0 as? CustomListSelectableLocationNode }
    }

    var iWasFirst = false

    var isCustomListSelected: Bool = false {
        didSet {
            if oldValue != isCustomListSelected {
                validateTree()
            }
        }
    }

    func validateTree() {
        let iWasReallyFirst = !root.flattened
            .compactMap {
                ($0 as? CustomListSelectableLocationNode)?._iWasFirst
            }.contains(true)
        guard iWasReallyFirst else { return }
        iWasFirst = true
        defer { iWasFirst = false }
        forEachDescendant {
            ($0 as? CustomListSelectableLocationNode)?.isCustomListSelected = isCustomListSelected
        }

        forEachAncestor {
            ($0 as? CustomListSelectableLocationNode)?.isCustomListSelected =
                $0.children
                .compactMap { $0 as? CustomListSelectableLocationNode }
                .allSatisfy { $0.isCustomListSelected }
        }
    }
}
