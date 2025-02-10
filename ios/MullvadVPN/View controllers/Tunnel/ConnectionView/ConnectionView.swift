//
//  ConnectionView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct ConnectionView: View {
    @ObservedObject var connectionViewModel: ConnectionViewViewModel
    @ObservedObject var indicatorsViewModel: FeatureIndicatorsViewModel

    @State private(set) var isExpanded = false

    @State private(set) var scrollViewHeight: CGFloat = 0
    var hasFeatureIndicators: Bool { !indicatorsViewModel.chips.isEmpty }
    var action: ButtonPanel.Action?

    var body: some View {
        VStack {
            Spacer()
                .accessibilityIdentifier(AccessibilityIdentifier.connectionView.asString)
            VStack(spacing: 16) {
                VStack(alignment: .leading, spacing: 0) {
                    HeaderView(viewModel: connectionViewModel, isExpanded: $isExpanded)
                        .padding(.bottom, 8)
                    Divider()
                        .background(UIColor.secondaryTextColor.color)
                        .padding(.bottom, 16)
                        .showIf(isExpanded)

                    ScrollView {
                        HStack {
                            VStack(alignment: .leading, spacing: 0) {
                                Text(LocalizedStringKey("Active features"))
                                    .font(.footnote.weight(.semibold))
                                    .foregroundStyle(UIColor.primaryTextColor.color.opacity(0.6))
                                    .showIf(isExpanded && hasFeatureIndicators)

                                ChipContainerView(
                                    viewModel: indicatorsViewModel,
                                    tunnelState: connectionViewModel.tunnelStatus.state,
                                    isExpanded: $isExpanded
                                )
                                .padding(.bottom, isExpanded ? 16 : 0)
                                .showIf(hasFeatureIndicators)

                                DetailsView(viewModel: connectionViewModel)
                                    .padding(.bottom, 8)
                                    .showIf(isExpanded)
                            }
                            Spacer()
                        }
                        .sizeOfView { size in
                            withAnimation {
                                scrollViewHeight = size.height
                            }
                        }
                    }
                    .frame(maxHeight: scrollViewHeight)
                    .apply {
                        if #available(iOS 16.4, *) {
                            $0.scrollBounceBehavior(.basedOnSize)
                        } else {
                            $0
                        }
                    }
                }
                .transformEffect(.identity)
                .animation(.default, value: hasFeatureIndicators)
                ButtonPanel(viewModel: connectionViewModel, action: action)
            }
            .padding(16)
            .background(BlurView(style: .dark))
            .cornerRadius(12)
            .padding(16)
            .onChange(of: connectionViewModel.showsConnectionDetails) { newValue in
                if !newValue {
                    withAnimation {
                        isExpanded = false
                    }
                }
            }
        }
    }
}

#Preview("ConnectionView (Indicators)") {
    ConnectionViewComponentPreview(showIndicators: true) { indicatorModel, viewModel, _ in
        ConnectionView(connectionViewModel: viewModel, indicatorsViewModel: indicatorModel)
    }
}

#Preview("ConnectionView (No indicators)") {
    ConnectionViewComponentPreview(showIndicators: false) { indicatorModel, viewModel, _ in
        ConnectionView(connectionViewModel: viewModel, indicatorsViewModel: indicatorModel)
    }
}
