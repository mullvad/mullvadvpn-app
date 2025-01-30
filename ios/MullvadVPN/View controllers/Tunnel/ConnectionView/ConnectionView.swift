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

    @State private(set) var showConnectionDetailsAnimated = false
    @State private(set) var isExpandedAnimatied = false
    @State private(set) var scrollViewHeight: CGFloat = 0

    var action: ButtonPanel.Action?

    var body: some View {
        Spacer()
            .accessibilityIdentifier(AccessibilityIdentifier.connectionView.asString)

        VStack(alignment: .leading, spacing: 16) {
            VStack(alignment: .leading, spacing: 16) {
                HeaderView(viewModel: connectionViewModel, isExpanded: $isExpanded)

                if showConnectionDetailsAnimated {
                    Divider()
                        .background(UIColor.secondaryTextColor.color)
                        .showIf(isExpandedAnimatied)

                    ScrollView {
                        VStack(alignment: .leading, spacing: 0) {
                            Text(LocalizedStringKey("Active features"))
                                .font(.footnote.weight(.semibold))
                                .foregroundStyle(UIColor.primaryTextColor.color.opacity(0.6))
                                .padding(.bottom, isExpandedAnimatied ? 8 : 0)
                                .showIf(!indicatorsViewModel.chips.isEmpty && isExpandedAnimatied)

                            ChipContainerView(viewModel: indicatorsViewModel, isExpanded: $isExpanded)

                            DetailsView(viewModel: connectionViewModel)
                                .padding(.top, indicatorsViewModel.chips.isEmpty ? 0 : 16)
                                .showIf(isExpandedAnimatied)
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
                    .showIf(isExpandedAnimatied || !indicatorsViewModel.chips.isEmpty)
                }
            }
            .transformEffect(.identity)

            ButtonPanel(viewModel: connectionViewModel, action: action)
                .background(
                    ZStack {
                        Rectangle().blendMode(.destinationOut)
                        BlurView(style: .dark)
                    }
                )
        }
        .padding()
        .background(BlurView(style: .dark))
        .cornerRadius(12)
        .padding()
        .onChange(of: isExpanded) { newValue in
            withAnimation {
                isExpandedAnimatied = newValue
            }
        }
        .onChange(of: connectionViewModel.showsConnectionDetails) { newValue in
            if !newValue {
                isExpanded = false
            }
            withAnimation {
                showConnectionDetailsAnimated = newValue
            }
        }
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
