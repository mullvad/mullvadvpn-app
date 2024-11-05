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

struct SingleChoiceList<T>: View where T: Hashable {
    let title: String
    let options: [T]
    var value: Binding<T>

    func row(_ v: T) -> some View {
        let isSelected = value.wrappedValue == v
        return HStack {
            Image("IconTick").opacity(isSelected ? 1.0 : 0.0)
            Text(verbatim: "\(v)")
            Spacer()
        }
        .padding(16)
        .background(isSelected ? Color(UIColor.Cell.Background.selected) : Color(UIColor.Cell.Background.normal))
        .foregroundColor(Color(UIColor.Cell.titleTextColor))
        .onTapGesture {
            value.wrappedValue = v
        }
    }

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Text(title)
                Spacer()
            }
            .padding(16)
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
