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

struct CustomListLocationItemView: View {
    @Bindable var node: CustomListSelectableLocationNode
    let level: Int

    var title: String {
        node.name
    }

    @ViewBuilder var statusIndicator: some View {
        let itemFactory = SegmentedListItemFactory()
        Group {
            itemFactory
                .statusIndicator(for: .checkbox(isOn: $node.isCustomListSelected))
            if !node.isActive {
                itemFactory
                    .statusIndicator(for: .dot(.offline))
            }
        }
    }

    var body: some View {
        ListItem(
            title: title,
            level: level,
            statusIndicator: { statusIndicator }
        )
    }
}

#Preview {
    @Previewable @State var node = CustomListSelectableLocationNode(
        fromLocationNode: LocationNode(
            name: "A great location",
            code: "a-great-location"
        ))

    CustomListLocationItemView(
        node: node,
        level: 0
    )
    .background(Color.MullvadList.Item.parent)
}
