//
//  ConnectionDetailsContainer.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2025-01-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

extension ConnectionView {
    internal struct DetailsContainer: View {
        @StateObject var viewModel: ConnectionViewViewModel
        @StateObject var indicatorsViewModel: FeatureIndicatorsViewModel
        @Binding var isExpanded: Bool

        @State private var scrollViewHeight: CGFloat = 0

        var body: some View {
            Divider()
                .background(UIColor.secondaryTextColor.color)
                .showIf(isExpanded)

            // This geometry reader is somewhat of a workaround. It's "smart" in that it takes up as much
            // space as it can and thereby helps the view to understand the maximum allowed height when
            // placed in a UIKit context. If ConnectionView would ever be placed as a subview of SwiftUI
            // parent, this reader could probably be removed.
            GeometryReader { _ in
                ScrollView {
                    VStack(spacing: 16) {
                        FeatureIndicatorsView(
                            viewModel: indicatorsViewModel,
                            isExpanded: $isExpanded
                        )
                        .showIf(!indicatorsViewModel.chips.isEmpty)

                        DetailsView(viewModel: viewModel)
                            .transition(.move(edge: .bottom))
                            .showIf(isExpanded)
                    }
                    .sizeOfView { scrollViewHeight = $0.height }
                }
            }
            .frame(maxHeight: scrollViewHeight)
        }
    }
}

#Preview {
    ConnectionViewComponentPreview(showIndicators: true, isExpanded: true) { indicatorModel, viewModel, isExpanded in
        ConnectionView.DetailsContainer(
            viewModel: viewModel,
            indicatorsViewModel: indicatorModel,
            isExpanded: isExpanded
        )
    }
}
