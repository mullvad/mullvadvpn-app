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
                .foregroundStyle(Color(uiColor: .primaryTextColor))
        }
        .padding(.horizontal, UIMetrics.padding8)
        .padding(.vertical, UIMetrics.padding4)
        .background(
            RoundedRectangle(cornerRadius: UIMetrics.ConnectionPanelView.cornerRadius)
                .stroke(
                    Color(uiColor: .primaryColor),
                    lineWidth: 1
                )
                .background(
                    RoundedRectangle(cornerRadius: UIMetrics.ConnectionPanelView.cornerRadius)
                        .fill(Color(uiColor: .secondaryColor))
                )
        )
    }
}

#Preview {
    ChipView(item: ChipModel(name: LocalizedStringKey("Example")))
}
