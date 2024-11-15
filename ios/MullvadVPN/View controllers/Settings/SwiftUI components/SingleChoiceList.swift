//
//  SingleChoiceList.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2024-11-06.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

/**
 A component presenting a vertical list in the Mullvad style for selecting a single item from a list.
 The items can be any Hashable type.
 */

struct SingleChoiceList<Item>: View where Item: Hashable {
    let title: String
    let options: [Item]
    var value: Binding<Item>
    let itemDescription: (Item) -> String

    init(title: String, options: [Item], value: Binding<Item>, itemDescription: ((Item) -> String)? = nil) {
        self.title = title
        self.options = options
        self.value = value
        self.itemDescription = itemDescription ?? { "\($0)" }
    }

    func row(_ v: Item) -> some View {
        let isSelected = value.wrappedValue == v
        return HStack {
            Image(uiImage: UIImage(resource: .iconTick)).opacity(isSelected ? 1.0 : 0.0)
            Spacer().frame(width: UIMetrics.SettingsCell.selectableSettingsCellLeftViewSpacing)
            Text(verbatim: itemDescription(v))
            Spacer()
        }
        .padding(EdgeInsets(UIMetrics.SettingsCell.layoutMargins))
        .background(
            isSelected
                ? Color(UIColor.Cell.Background.selected)
                : Color(UIColor.Cell.Background.indentationLevelOne)
        )
        .foregroundColor(Color(UIColor.Cell.titleTextColor))
        .onTapGesture {
            value.wrappedValue = v
        }
    }

    var body: some View {
        VStack(spacing: UIMetrics.TableView.separatorHeight) {
            HStack {
                Text(title).fontWeight(.semibold)
                Spacer()
            }
            .padding(EdgeInsets(UIMetrics.SettingsCell.layoutMargins))
            .background(Color(UIColor.Cell.Background.normal))
            ForEach(options, id: \.self) { opt in
                row(opt)
            }
            Spacer()
        }
        .background(Color(.secondaryColor))
        .foregroundColor(Color(.primaryTextColor))
    }
}

#Preview {
    StatefulPreviewWrapper(1) { SingleChoiceList(title: "Test", options: [1, 2, 3], value: $0) }
}
