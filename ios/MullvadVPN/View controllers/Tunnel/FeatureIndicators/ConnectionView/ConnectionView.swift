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

    var action: ButtonPanel.Action?
    var onContentUpdate: (() -> Void)?

    var body: some View {
        Spacer()
            .accessibilityIdentifier(AccessibilityIdentifier.connectionView.asString)

        VStack(spacing: 22) {
            CustomProgressView(style: .large)
                .showIf(connectionViewModel.showsActivityIndicator)

            ZStack {
                BlurView(style: .dark)

                VStack(alignment: .leading, spacing: 0) {
                    HeaderView(viewModel: connectionViewModel, isExpanded: $isExpanded)
                        .padding(.bottom, headerViewBottomPadding)

                    DetailsContainer(
                        connectionViewModel: connectionViewModel,
                        indicatorsViewModel: indicatorsViewModel,
                        isExpanded: $isExpanded
                    )
                    .showIf(connectionViewModel.showConnectionDetails)

                    ButtonPanel(viewModel: connectionViewModel, action: action)
                        .padding(.top, 16)
                }
                .padding(16)
            }
            .cornerRadius(12)
            .padding(16)
        }
        .padding(.bottom, 8) // Some spacing to avoid overlap with the map legal link.
        .onChange(of: isExpanded) { _ in
            onContentUpdate?()
        }
        .onReceive(connectionViewModel.combinedState) { _, _ in
            // Only update expanded state when connections details should be hidden.
            // This will contract the view on eg. disconnect, but leave it as-is on
            // eg. connect.
            if !connectionViewModel.showConnectionDetails {
                isExpanded = false
                return
            }

            onContentUpdate?()
        }
    }
}

extension ConnectionView {
    var headerViewBottomPadding: CGFloat {
        let showConnectionDetails = connectionViewModel.showConnectionDetails
        let hasIndicators = !indicatorsViewModel.chips.isEmpty

        return isExpanded
            ? 16
            : hasIndicators && showConnectionDetails ? 16 : 0
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
