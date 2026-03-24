//
//  RecentLocationListItem.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-11-27.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
import SwiftUI

struct RecentLocationListItem<ContextMenu>: View where ContextMenu: View {
    @State private var alert: MullvadAlert?
    private let itemFactory = ListItemFactory()

    @Binding var location: LocationNode
    let onSelect: (LocationNode) -> Void
    let contextMenu: (LocationNode) -> ContextMenu

    var body: some View {
        if location is AutomaticLocationNode {
            AutomaticLocationListItem(location: $location, isRecent: true, onSelect: onSelect)
        } else {
            recentLocationListItem
        }
    }

    @ViewBuilder
    var recentLocationListItem: some View {
        SegmentedListItem(
            accessibilityIdentifier: .recentListItem(location.name),
            accessibilityLabel: location.name,
            label: {
                itemFactory.label(for: .recent(node: location))
            },
            segment: {},
            groupedContent: {},
            onSelect: {
                onSelect(location)
            }
        )
        .contextMenu {
            contextMenu(location)
        }
        .id(location.id)  // to be able to scroll to this item programmatically
        .mullvadAlert(item: $alert)
    }
}
