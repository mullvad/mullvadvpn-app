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
                        .padding(.bottom, 4)

                    Divider()
                        .background(UIColor.secondaryTextColor.color)
                        .padding(.top, 4)
                        .padding(.bottom, 8)
                        .showIf(isExpanded)

                    ScrollView {
                        VStack(alignment: .leading, spacing: 2) {
                            if let titleForCountryAndCity = connectionViewModel.titleForCountryAndCity {
                                Text(titleForCountryAndCity)
                                    .lineLimit(isExpanded ? 2 : 1)
                                    .font(.title3.weight(.semibold))
                                    .foregroundStyle(UIColor.primaryTextColor.color)
                            }
                            if let titleForServer = connectionViewModel.titleForServer {
                                Text(titleForServer)
                                    .lineLimit(isExpanded ? 3 : 1)
                                    .font(.body)
                                    .foregroundStyle(UIColor.primaryTextColor.color.opacity(0.6))
                                    .accessibilityIdentifier(
                                        AccessibilityIdentifier.connectionPanelServerLabel.asString
                                    )
                                    .multilineTextAlignment(.leading)
                                    .fixedSize(horizontal: false, vertical: true)
                            }
                            HStack {
                                VStack(alignment: .leading, spacing: 0) {
                                    Text(LocalizedStringKey("Active features"))
                                        .font(.footnote.weight(.semibold))
                                        .foregroundStyle(UIColor.primaryTextColor.color.opacity(0.6))
                                        .padding(.top, 8)
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
                                        .padding(.top, !hasFeatureIndicators ? 8 : 0)
                                        .showIf(isExpanded)
                                }
                                Spacer()
                            }
                        }.frame(maxWidth: .infinity, alignment: .leading)
                            .sizeOfView { size in
                                withAnimation {
                                    scrollViewHeight = size.height
                                }
                            }
                    }
                    .frame(maxHeight: scrollViewHeight)
                    .apply {
                        $0.scrollBounceBehavior(.basedOnSize)
                    }
                }
                .transformEffect(.identity)
                .animation(.default, value: hasFeatureIndicators)
                ButtonPanel(viewModel: connectionViewModel, action: action)
            }
            .padding(16)
            .background(BlurView(style: .dark))
            .cornerRadius(12)
            .padding(EdgeInsets(top: 16, leading: 16, bottom: 24, trailing: 16))
            .onChange(
                of: connectionViewModel.showsConnectionDetails,
                { oldValue, newValue in
                    if oldValue != newValue {
                        withAnimation {
                            isExpanded = false
                        }
                    }
                })
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
