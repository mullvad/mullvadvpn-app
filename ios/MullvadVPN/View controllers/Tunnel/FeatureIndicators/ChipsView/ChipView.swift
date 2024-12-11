//
//  FeatureChipView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-12-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct ChipView: View {
    let item: ChipModel
    var body: some View {
        HStack(spacing: UIMetrics.padding4) {
            Text(item.name)
                .font(.body)
                .lineLimit(1)
                .multilineTextAlignment(.center)
                .foregroundStyle(UIColor.primaryTextColor.color)
        }
        .padding(.horizontal, UIMetrics.padding8)
        .padding(.vertical, UIMetrics.padding4)
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
    ChipView(item: ChipModel(name: LocalizedStringKey("Example")))
}
