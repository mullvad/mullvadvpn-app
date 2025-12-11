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
        // Get the full path to this node and drop the last element (which is the node itself)
        if let parent = location.parent,
            let locations = parent.root.pathToNode(matchingCode: parent.code),
            !locations.dropLast().isEmpty
        {
            return "\(locations.dropLast().joined(separator: ", "))"
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

extension LocationNode {
    func pathToNode(matchingCode code: String) -> [String]? {
        if self.code == code {
            return [NSLocalizedString(name, comment: "")]
        }
        for child in children {
            if let childPath = child.pathToNode(matchingCode: code) {
                return [NSLocalizedString(name, comment: "")] + childPath
            }
        }
        return nil
    }
}
