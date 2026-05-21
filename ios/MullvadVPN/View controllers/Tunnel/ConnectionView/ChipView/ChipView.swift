//
//  FeatureChipView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-12-05.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct ChipView: View {
    enum Style {
        case mainScreen
        case dialog
    }

    let item: ChipModel
    let style: Style
    let onPress: (() -> Void)?
    private let borderWidth: CGFloat = 1

    var backgroundColor: Color {
        switch style {
        case .mainScreen: UIColor.secondaryColor.color
        case .dialog: UIColor.primaryColor.color
        }
    }

    var body: some View {
        Button {
            onPress?()
        } label: {
            HStack(spacing: UIMetrics.FeatureIndicators.chipViewIconTextSpacing) {
                if let icon = item.icon {
                    icon
                        .resizable()
                        .frame(width: 14, height: 14)
                }
                Text(item.name)
                    .font(.subheadline)
                    .lineLimit(1)
                    .foregroundStyle(UIColor.primaryTextColor.color)
                    .padding(.vertical, 4)
            }
            .padding(.horizontal, UIMetrics.FeatureIndicators.chipViewHorizontalPadding)
        }
        .background(
            RoundedRectangle(cornerRadius: 8)
                .stroke(
                    UIColor.primaryColor.color,
                    lineWidth: borderWidth
                )
                .background(
                    RoundedRectangle(cornerRadius: 8)
                        .fill(backgroundColor)
                )
                .padding(borderWidth)
        )
    }
}

#Preview("Text only, main style") {
    ZStack {
        ChipView(item: ChipModel(id: .daita, name: "Example"), style: .mainScreen) {}
    }
    .frame(maxWidth: .infinity, maxHeight: .infinity)
    .background(UIColor.secondaryColor.color)
}

#Preview("Text only, dialog style") {
    ZStack {
        ChipView(item: ChipModel(id: .daita, name: "Example"), style: .dialog) {}
    }
    .frame(maxWidth: .infinity, maxHeight: .infinity)
    .background(Color.mullvadBackground)
}

#Preview("Text + icon") {
    ZStack {
        ChipView(item: ChipModel(id: .daita, name: "Example", icon: .mullvadIconMultihopWhenNeeded), style: .mainScreen)
        {}
    }
    .frame(maxWidth: .infinity, maxHeight: .infinity)
    .background(UIColor.secondaryColor.color)
}
