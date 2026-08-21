// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadTypes
import SwiftUI

struct RecentLocationListItem<ContextMenu>: View where ContextMenu: View {
    @State private var alert: MullvadAlert?
    private let itemFactory = SegmentedListItemFactory()

    @Binding var location: LocationNode
    let multihopContext: MultihopContext
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
        Color.clear.frame(height: 4).accessibilityHidden(true)

        SegmentedListItem(
            userInteraction: location.isExcluded ? .disabled : .enabled,
            accessibilityIdentifier: .recentListItem(location.name),
            accessibilityLabel: location.name,
            leading: {
                itemFactory.leading(for: .recentLocation(node: location, context: multihopContext))
            },
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
