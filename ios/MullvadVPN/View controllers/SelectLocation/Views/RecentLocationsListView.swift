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

    var filteredLocationIndices: [Int] {
        locations
            .enumerated()
            .map { $0.offset }
    }

    var body: some View {
        ForEach(
            Array(filteredLocationIndices.enumerated()),
            id: \.element
        ) { index, indexInLocationList in
            let location = $locations[indexInLocationList]
            RecentLocationListItem(
                location: location,
                onSelect: onSelectLocation,
                contextMenu: { location in contextMenu(location) },
            )
        }
    }
}
