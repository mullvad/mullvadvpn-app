//
//  RecentLocationsListView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-11-27.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import SwiftUI

struct RecentLocation {
    var node: LocationNode
    let info: LocalizedStringKey?
}
struct RecentLocationsListView<ContextMenu>: View where ContextMenu: View {
    @Binding var locations: [RecentLocation]
    let multihopContext: MultihopContext
    let onSelectLocation: (LocationNode) -> Void
    let contextMenu: (LocationNode) -> ContextMenu

    var filteredLocationIndices: [Int] {
        locations
            .enumerated()
            .filter { $0.element.node.isHiddenFromSearch }
            .map { $0.offset }
    }

    var body: some View {
        ForEach(
            Array(filteredLocationIndices.enumerated()),
            id: \.element
        ) { index, indexInLocationList in
            let location = $locations[indexInLocationList]
            RecentLocationListItem(
                location: location.node,
                multihopContext: multihopContext,
                subtitle: location.wrappedValue.info,
                onSelect: onSelectLocation,
                contextMenu: { location in contextMenu(location) },
            )
            .padding(.bottom, index == filteredLocationIndices.count - 1 ? 24 : 0)
        }
    }
}

//var subtitle: LocalizedStringKey? {
//    // Get the full path to this node and drop the last element (which is the node itself)
//    if let parent = location.parent,
//        let locations = parent.root.pathToNode(matchingCode: parent.code),
//        !locations.dropLast().isEmpty
//    {
//        return "\(locations.dropLast().joined(separator: ", "))"
//    }
//    return nil
//}
