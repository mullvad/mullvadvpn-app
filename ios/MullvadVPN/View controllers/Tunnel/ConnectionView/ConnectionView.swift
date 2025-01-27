//
//  ConnectionView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-03.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
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
        VStack {
            VStack(alignment: .leading, spacing: 0) {
                HeaderView(viewModel: connectionViewModel, isExpanded: $isExpanded)
                    .padding(.bottom, !indicatorsViewModel.chips.isEmpty || isExpandedAnimatied ? 16 : 0)
                if showConnectionDetailsAnimated {
                    if isExpandedAnimatied {
                        Divider()
                            .background(UIColor.secondaryTextColor.color)
                            .transition(.offset(y: scrollViewHeight).combined(with: .opacity.combined(with: .scale(
                                scale: 0,
                                anchor: .center
                            ))))
                            .padding(.bottom, 16)
                    }
                    ScrollView {
                        VStack(alignment: .leading) {
                            if !indicatorsViewModel.chips.isEmpty && isExpandedAnimatied {
                                Text(LocalizedStringKey("Active features"))
                                    .font(.footnote.weight(.semibold))
                                    .foregroundStyle(UIColor.primaryTextColor.color.opacity(0.6))
                                    .padding(.bottom, isExpandedAnimatied ? 16 : 0)
                            }
                            ChipContainerView(viewModel: indicatorsViewModel, isExpanded: $isExpanded)
                            if isExpandedAnimatied {
                                DetailsView(viewModel: connectionViewModel)
                                    .padding(.top, indicatorsViewModel.chips.isEmpty ? 0 : 16)
                            }
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
                ButtonPanel(viewModel: connectionViewModel, action: action)
                    .padding(.top, 16)
            }
            .padding()
        }
        .onChange(of: isExpanded) { newValue in
            withAnimation {
                isExpandedAnimatied = newValue
            }
        }
        .onChange(of: connectionViewModel.showConnectionDetails) { newValue in
            withAnimation {
                showConnectionDetailsAnimated = newValue
            }
            if !newValue {
                withAnimation {
                    isExpandedAnimatied = false
                }
            }
        }
        .background(BlurView(style: .dark))
        .cornerRadius(12)
        .padding()
        .onReceive(connectionViewModel.$tunnelStatus) { _ in
            // Only update expanded state when connections details should be hidden.
            // This will contract the view on eg. disconnect, but leave it as-is on
            // eg. connect.
            if !connectionViewModel.showConnectionDetails {
                isExpanded = false
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
