//
//  FeatureChipView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-12-05.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct DashedBorderView: View {
    var body: some View {
        Rectangle()
            .fill(Color.clear)
            .frame(width: 100, height: 100)
            .overlay(
                RoundedRectangle(cornerRadius: 10)
                    .strokeBorder(style: StrokeStyle(lineWidth: 2, dash: [10, 5]))
            )
    }
}

struct ChipView: View {
    let item: ChipModel
    let onPress: (() -> Void)?
    private let borderWidth: CGFloat = 1

    var body: some View {
        Button {
            onPress?()
        } label: {
            Text(item.name)
                .font(.subheadline)
                .lineLimit(1)
                .foregroundStyle(UIColor.primaryTextColor.color)
                .padding(.horizontal, UIMetrics.FeatureIndicators.chipViewHorisontalPadding)
                .padding(.vertical, 4)
                .background(
                    RoundedRectangle(cornerRadius: 8)
                        .strokeBorder(style: StrokeStyle(lineWidth: borderWidth, dash: item.isMultihopEverywhere ? [10, 5] : []))
                        .foregroundStyle(UIColor.primaryColor.color)
//                        .stroke(
//                            UIColor.primaryColor.color,
//                            lineWidth: borderWidth
//                        )
                        .background(
                            RoundedRectangle(cornerRadius: 8)
                                .fill(UIColor.secondaryColor.color)
                        )
                        .padding(borderWidth)
                )
        }
    }
}

#Preview {
    ZStack {
        ChipView(item: ChipModel(id: .daita, name: "Example", isMultihopEverywhere: false)) {}
    }
    .frame(maxWidth: .infinity, maxHeight: .infinity)
    .background(UIColor.secondaryColor.color)
}
