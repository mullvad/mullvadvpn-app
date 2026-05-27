//
//  RecentLocationView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-03-20.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

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
        let itemFactory = ListItemFactory()

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
