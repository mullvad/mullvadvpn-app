// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import SwiftUI

struct ChipView: View {
    let item: ChipModel
    let onPress: (() -> Void)?
    private let borderWidth: CGFloat = 1

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
                        .fill(UIColor.secondaryColor.color)
                )
                .padding(borderWidth)
        )
        .accessibilityIdentifier(item.id.accessibilityId)
        .accessibilityLabel(item.name)
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
        ChipView(item: ChipModel(id: .daita, name: "Example", icon: .mullvadIconMultihopWhenNeeded)) {}
    }
    .frame(maxWidth: .infinity, maxHeight: .infinity)
    .background(UIColor.secondaryColor.color)
}
