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
    let onSelect: (LocationNode) -> Void
    let contextMenu: (LocationNode) -> ContextMenu

    var subtitle: LocalizedStringKey? {
        if let parent = location.parent {
            let root = parent.root
            if root == parent {
                return LocalizedStringKey(parent.name)
            }
            return "\(root.name), \(parent.name)"
        }
        return nil
    }

    var body: some View {
        RelayItemView(
            location: location,
            multihopContext: multihopContext,
            level: 0,
            subtitle: subtitle,
            isLastInList: true,
            onSelect: { onSelect(location) }
        )
        .accessibilityIdentifier(.locationListItem(location.name))
        .contextMenu {
            contextMenu(location)
        }
        .padding(.top, 4)
        .id(location.code)  // to be able to scroll to this item programmatically
    }
}
