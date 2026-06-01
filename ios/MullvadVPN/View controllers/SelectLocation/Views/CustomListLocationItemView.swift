//
//  CustomListLocationItemView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-05-19.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

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
