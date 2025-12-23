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
            .filter { $0.element.isHiddenFromSearch }
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
                multihopContext: multihopContext,
                subtitle: getSubtitle(location.wrappedValue),
                onSelect: onSelectLocation,
                contextMenu: { location in contextMenu(location) },
            )
            .padding(.bottom, index == filteredLocationIndices.count - 1 ? 24 : 0)
        }
    }

    func getSubtitle(_ location: LocationNode) -> LocalizedStringKey? {
        guard let recentLocationNode = location.asRecentLocationNode,
            recentLocationNode.parent?.asCustomListNode == nil,
            let ancestors = recentLocationNode.locationInfo?.dropLast(),
            ancestors.isEmpty == false
        else {
            return nil
        }
        return "\(ancestors.joined(separator: ", "))"
    }
}
