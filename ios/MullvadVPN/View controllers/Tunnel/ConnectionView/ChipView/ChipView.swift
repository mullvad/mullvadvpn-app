//
//  FeatureChipView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-12-05.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct ChipView: View {
    let item: ChipModel
    private let borderWidth: CGFloat = 1

    var body: some View {
        Text(item.name)
            .font(.subheadline)
            .lineLimit(1)
            .foregroundStyle(UIColor.primaryTextColor.color)
            .padding(.horizontal, UIMetrics.FeatureIndicators.chipViewHorisontalPadding)
            .padding(.vertical, 4)
            .background(
                RoundedRectangle(cornerRadius: 8)
                    .stroke(
                        UIColor.primaryColor.color,
                        lineWidth: borderWidth
                    )
                    .background(
                        RoundedRectangle(cornerRadius: 8)
                            .fill(UIColor.secondaryColor.color)
                    )
                    .padding(borderWidth)
            )
    }
}

#Preview {
    ZStack {
        ChipView(item: ChipModel(name: "Example"))
    }
    .frame(maxWidth: .infinity, maxHeight: .infinity)
    .background(UIColor.secondaryColor.color)
}
