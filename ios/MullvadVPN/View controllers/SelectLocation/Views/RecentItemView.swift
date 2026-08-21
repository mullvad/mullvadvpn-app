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
import SwiftUI

struct RecentItemView: View {
    let node: LocationNode
    let multihopContext: MultihopContext

    var title: String {
        if node.isExcluded {
            switch multihopContext {
            case .entry:
                return """
                    \(node.name) (\(String(localized:
                    String
                    .LocalizationValue(MultihopContext.exit.description))))
                    """
            case .exit:
                return """
                    \(node.name) (\(String(localized:
                    String
                    .LocalizationValue(MultihopContext.entry.description))))
                    """
            }
        }
        return "\(node.name)"
    }

    var subtitle: String? {
        var ancestors: [String] = []

        if let recentLocationNode = node.asRecentLocationNode,
            (node.userSelectedRelays.customListSelection?.isList ?? false) == false
        {
            ancestors = recentLocationNode.locationInfo?.dropLast() ?? []
        } else if let automaticLocationNode = node.asAutomaticLocationNode {
            ancestors = automaticLocationNode.locationInfo ?? []
        }

        return ancestors.isEmpty ? nil : ancestors.joined(separator: ", ")
    }

    @ViewBuilder var statusIndicator: some View {
        let itemFactory = SegmentedListItemFactory()

        if node.isSelected {
            itemFactory.statusIndicator(for: .tick)
        } else {
            EmptyView()
        }
    }

    var body: some View {
        ListItem(
            title: title,
            subtitle: subtitle,
            level: 0,
            selected: node.isSelected,
            statusIndicator: { statusIndicator }
        )
        .disabled(node.isExcluded)
    }
}

#Preview {
    RecentItemView(
        node: LocationNode(
            name: "A great location",
            code: "a-great-location",
            isSelected: true
        ),
        multihopContext: .exit
    )
}
