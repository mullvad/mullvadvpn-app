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

        @State var showMoreDetails = false

        @State private var scrollViewHeight: CGFloat = 0
        var body: some View {
            VStack(spacing: 16) {
                Divider()
                    .background(UIColor.secondaryTextColor.color)
                    .showIf(showMoreDetails)
                    .transition(.opacity.combined(with: .offset(y: 200)))
                ScrollView {
                    VStack(spacing: 16) {
                        FeatureIndicatorsView(
                            viewModel: indicatorsViewModel,
                            isExpanded: $isExpanded
                        )
                        .background(Color.green)
                        .showIf(!indicatorsViewModel.chips.isEmpty)

                        DetailsView(viewModel: connectionViewModel)
                            .showIf(showMoreDetails)
                            .transition(.opacity.combined(with: .offset(y: 200)))
                    }
                    .sizeOfView { view in
                        withAnimation(.default) {
                            scrollViewHeight = view.height
                        }
                    }
                }
                .showIf(showMoreDetails || !indicatorsViewModel.chips.isEmpty)
                .frame(minHeight: 0, maxHeight: scrollViewHeight)
                .onTapGesture {
                    // If this callback is not set the child views will not reliably register tap events.
                    // This is a bug in iOS 16 and 17, but seemingly fixed in 18. Once we set the lowest
                    // supported version to iOS 18 we can probably remove it.
                }
                .apply {
                    if #available(iOS 16.4, *) {
                        $0.scrollBounceBehavior(.basedOnSize)
                    } else {
                        $0
                    }
                }
            }
            .onChange(of: isExpanded) { newValue in
                withAnimation {
                    showMoreDetails = newValue
                }
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

#Preview("ConnectionView (Indicators)") {
    ConnectionViewComponentPreview(showIndicators: true, isExpanded: true) { indicatorModel, viewModel, _ in
        ConnectionView(connectionViewModel: viewModel, indicatorsViewModel: indicatorModel)
    }
}

#Preview("ConnectionView (No indicators)") {
    ConnectionViewComponentPreview(showIndicators: false, isExpanded: true) { indicatorModel, viewModel, _ in
        ConnectionView(connectionViewModel: viewModel, indicatorsViewModel: indicatorModel)
    }
}
