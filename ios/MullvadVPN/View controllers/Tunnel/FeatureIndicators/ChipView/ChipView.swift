//
//  ChipView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-12-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct ChipView: View {
    let item: ChipModel
    var body: some View {
        Text(item.name)
            .font(.subheadline)
            .lineLimit(1)
            .foregroundStyle(UIColor.primaryTextColor.color)
            .padding(.horizontal, 8)
            .padding(.vertical, 4)
            .background(
                RoundedRectangle(cornerRadius: 8.0)
                    .stroke(
                        UIColor.primaryColor.color,
                        lineWidth: 1
                    )
                    .background(
                        RoundedRectangle(cornerRadius: 8.0)
                            .fill(UIColor.secondaryColor.color)
                    )
            )
    }
}

#Preview {
    ZStack {
        ChipView(item: ChipModel(name: LocalizedStringKey("Example")))
    }
    .frame(maxWidth: .infinity, maxHeight: .infinity)
    .background(UIColor.secondaryColor.color)
}
