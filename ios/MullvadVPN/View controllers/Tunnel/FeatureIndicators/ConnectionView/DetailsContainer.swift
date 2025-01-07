//
//  DetailsContainer.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2025-01-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

extension ConnectionView {
    internal struct DetailsContainer: View {
        @ObservedObject var connectionViewModel: ConnectionViewViewModel
        @ObservedObject var indicatorsViewModel: FeatureIndicatorsViewModel
        @Binding var isExpanded: Bool

        @State private var scrollViewHeight: CGFloat = 0

        var body: some View {
            VStack(spacing: 16) {
                Divider()
                    .background(UIColor.secondaryTextColor.color)
                    .showIf(isExpanded)

                ScrollView {
                    VStack(spacing: 16) {
                        FeatureIndicatorsView(
                            viewModel: indicatorsViewModel,
                            isExpanded: $isExpanded
                        )
                        .showIf(!indicatorsViewModel.chips.isEmpty)

                        DetailsView(viewModel: connectionViewModel)
                            .showIf(isExpanded)
                    }
                    .sizeOfView { scrollViewHeight = $0.height }
                }
                .frame(maxHeight: scrollViewHeight)
            }
        }
    }
}

#Preview {
    ConnectionViewComponentPreview(showIndicators: true, isExpanded: true) { indicatorModel, viewModel, isExpanded in
        ConnectionView.DetailsContainer(
            connectionViewModel: viewModel,
            indicatorsViewModel: indicatorModel,
            isExpanded: isExpanded
        )
    }
}
