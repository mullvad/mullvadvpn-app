//
//  RecentLocationsListView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-11-27.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import MullvadTypes
import SwiftUI

struct RecentLocationsListView<ContextMenu>: View where ContextMenu: View {
    @Binding var locations: [LocationNode]
    let multihopContext: MultihopContext
    let onSelectLocation: (LocationNode) -> Void
    let contextMenu: (LocationNode) -> ContextMenu

    var body: some View {
        ForEach($locations, id: \.self) { location in
            RecentLocationListItem(
                location: location,
                multihopContext: multihopContext,
                onSelect: onSelectLocation,
                contextMenu: { location in contextMenu(location) },
            )
        }
    }
}
