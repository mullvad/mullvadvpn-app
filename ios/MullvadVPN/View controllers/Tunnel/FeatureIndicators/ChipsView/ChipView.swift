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
                .foregroundStyle(Color(uiColor: .primaryTextColor))
        }
        .padding(.horizontal, UIMetrics.padding8)
        .padding(.vertical, UIMetrics.padding4)
        .background(Color(uiColor: .secondaryColor))
        .overlay {
            RoundedRectangle(cornerRadius: UIMetrics.controlCornerRadius)
                .stroke(Color(uiColor: .primaryColor), style: StrokeStyle(lineWidth: 1.0))
        }
    }
}

#Preview {
    ChipView(item: ChipModel(name: LocalizedStringKey("Example")))
}
