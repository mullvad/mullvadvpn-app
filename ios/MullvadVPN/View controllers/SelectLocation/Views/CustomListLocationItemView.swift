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
    let node: LocationNode
    let level: Int
    @Binding var isSelected: Bool

    var title: String {
        node.name
    }

    var isDisabled: Bool {
        !node.isActive
    }

    @ViewBuilder var statusIndicator: some View {
        let itemFactory = SegmentedListItemFactory()
        itemFactory.statusIndicator(for: .checkbox(isOn: $isSelected))
    }

    var body: some View {
        ListItem(
            title: title,
            level: level,
            statusIndicator: { statusIndicator }
        )
        .disabled(isDisabled)
    }
}

#Preview {
    @Previewable @State var isSelected: Bool = false

    CustomListLocationItemView(
        node: LocationNode(
            name: "A great location",
            code: "a-great-location"
        ),
        level: 0,
        isSelected: $isSelected
    )
    .background(Color.MullvadList.Item.parent)
}
