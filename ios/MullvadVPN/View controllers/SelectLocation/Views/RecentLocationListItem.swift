//
//  RecentLocationListItem.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-11-27.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct RecentLocationListItem<ContextMenu>: View where ContextMenu: View {
    @Binding var location: LocationNode
    let multihopContext: MultihopContext
    let subtitle: LocalizedStringKey?
    let onSelect: (LocationNode) -> Void
    let contextMenu: (LocationNode) -> ContextMenu

    var body: some View {
        RelayItemView(
            location: location,
            multihopContext: multihopContext,
            level: 0,
            subtitle: subtitle,
            isLastInList: true,
            onSelect: { onSelect(location) }
        )
        .accessibilityIdentifier(.recentListItem(location.name))
        .contextMenu {
            contextMenu(location)
        }
        .padding(.top, 4)
        .id(location.id)  // to be able to scroll to this item programmatically
    }
}
