//
//  RecentLocationsListView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-11-27.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import SwiftUI

struct RecentLocationsListView<ContextMenu>: View where ContextMenu: View {
    @Binding var locations: [LocationNode]
    let multihopContext: MultihopContext
    let onSelectLocation: (LocationNode) -> Void
    let contextMenu: (LocationNode) -> ContextMenu

    var filteredLocationIndices: [Int] {
        locations
            .enumerated()
            .filter { !$0.element.isHiddenFromSearch }
            .map { $0.offset }
    }

    var body: some View {
        ForEach(
            Array(filteredLocationIndices.enumerated()),
            id: \.element
        ) {
            index,
            indexInLocationList in
            let location = $locations[indexInLocationList]
            RecentLocationListItem(
                location: location,
                multihopContext: multihopContext,
                onSelect: onSelectLocation,
                contextMenu: { location in contextMenu(location) }
            )
        }
    }
}

#Preview {
    @Previewable @StateObject var viewModel = MockSelectLocationViewModel()
    ScrollView {
        LazyVStack(spacing: 0) {
            RecentLocationsListView(
                locations: $viewModel.exitContext.customLists,
                multihopContext: .exit,
                onSelectLocation: { location in
                    print("Selected: \(location.name)")
                },
                contextMenu: { location in Text("Add \(location.name) to list") }
            )
            .padding(.horizontal)
        }
    }
    .background(Color.mullvadBackground)
}
