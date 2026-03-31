//
//  FeatureChipView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-12-05.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct ChipView: View {
    let item: ChipModel
    let onPress: (() -> Void)?
    private let borderWidth: CGFloat = 1

    private var content: Text {
        if let icon = item.icon {
            Text("\(icon) \(item.name)")
        } else {
            Text(item.name)
        }

    }

    var body: some View {
        Button {
            onPress?()
        } label: {
            content
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
}

#Preview("Text only") {
    ZStack {
        ChipView(item: ChipModel(id: .daita, name: "Example")) {}
    }
    .frame(maxWidth: .infinity, maxHeight: .infinity)
    .background(UIColor.secondaryColor.color)
}

#Preview("Text + icon") {
    ZStack {
        ChipView(item: ChipModel(id: .daita, name: "Example", icon: Image("IconSmartLocation"))) {}
    }
    .frame(maxWidth: .infinity, maxHeight: .infinity)
    .background(UIColor.secondaryColor.color)
}
