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

    private var standardBackground: some View {
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
    }

    var body: some View {
        let chip = Button {
            onPress?()
        } label: {
            HStack(spacing: UIMetrics.FeatureIndicators.chipViewIconTextSpacing) {
                if let icon = item.icon {
                    icon
                        .foregroundStyle(UIColor.primaryTextColor.color)
                }
                Text(item.name)
                    .font(.subheadline)
                    .lineLimit(1)
                    .foregroundStyle(UIColor.primaryTextColor.color)
                    .padding(.vertical, 4)
            }
            .padding(.horizontal, UIMetrics.FeatureIndicators.chipViewHorizontalPadding)
        }
        .background(standardBackground)

        #if NEVER_IN_PRODUCTION
        if item.style == .rainbowShimmer {
            chip.gotaTunStyle()
        } else {
            chip
        }
        #else
        chip
        #endif
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
