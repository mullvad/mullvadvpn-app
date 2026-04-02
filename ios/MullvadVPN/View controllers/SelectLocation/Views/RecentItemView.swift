//
//  RecentLocationView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-03-20.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct RecentItemView: View {
    let node: LocationNode

    var isDisabled: Bool {
        !node.isActive || node.isExcluded
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
            title: node.name,
            subtitle: subtitle,
            level: 0,
            selected: node.isSelected,
            statusIndicator: { statusIndicator }
        )
        .disabled(isDisabled)
    }
}

#Preview {
    RecentItemView(
        node: LocationNode(
            name: "A great location",
            code: "a-great-location",
            isSelected: true
        )
    )
}
