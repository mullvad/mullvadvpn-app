//
//  FeaturesIndicatorsView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-12-06.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct FeatureIndicatorsView<ViewModel>: View where ViewModel: ChipViewModelProtocol {
    @ObservedObject var viewModel: ViewModel
    @Binding var isExpanded: Bool

    @State private var showExpandedText: Bool = false
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(LocalizedStringKey("Active features"))
                .font(.footnote.weight(.semibold))
                .foregroundStyle(UIColor.primaryTextColor.color.opacity(0.6))
                .showIf(showExpandedText)
                .apply {
                    if #available(iOS 16.0, *) {
                        $0.transition(.push(from: .bottom))
                    } else {
                        $0.transition(.opacity.combined(with: .move(edge: .bottom)))
                    }
                }
                .onChange(of: isExpanded) { newValue in
                    withAnimation {
                        showExpandedText = newValue
                    }
                }

            ChipContainerView(viewModel: viewModel, isExpanded: $isExpanded)
        }
    }
}

#Preview {
    FeatureIndicatorsView(
        viewModel: MockFeatureIndicatorsViewModel(),
        isExpanded: .constant(true)
    )
    .background(UIColor.secondaryColor.color)
}
