//
//  ConnectionView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-03.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct ConnectionView: View {
    @StateObject var connectionViewModel: ConnectionViewViewModel
    @StateObject var indicatorsViewModel: FeatureIndicatorsViewModel

    @State private(set) var isExpanded = false

    var action: ButtonPanel.Action?
    var onContentUpdate: (() -> Void)?

    var body: some View {
        Spacer()
        VStack(spacing: 22) {
            if connectionViewModel.showsActivityIndicator {
                CustomProgressView(style: .large)
            }

            ZStack {
                BlurView(style: .dark)

                VStack(alignment: .leading, spacing: 16) {
                    HeaderView(viewModel: connectionViewModel, isExpanded: $isExpanded)

                    if connectionViewModel.showConnectionDetails {
                        DetailsContainer(
                            viewModel: connectionViewModel,
                            indicatorsViewModel: indicatorsViewModel,
                            isExpanded: $isExpanded
                        )
                    }

                    ButtonPanel(viewModel: connectionViewModel, action: action)
                }
                .padding(16)
            }
            .cornerRadius(12)
            .padding(16)
        }
        .padding(.bottom, 8) // Adding some spacing so as not to overlap with the map legal link.
        .accessibilityIdentifier(AccessibilityIdentifier.connectionView.asString)
        .onChange(of: isExpanded) { _ in
            onContentUpdate?()
        }
        .onReceive(connectionViewModel.combinedState) { _, _ in
            onContentUpdate?()

            if !connectionViewModel.showConnectionDetails {
//                withAnimation {
                isExpanded = false
//                }
            }
        }
    }
}

#Preview("ConnectionView (Indicators)") {
    ConnectionViewPreview(configuration: .normal).make()
}

#Preview("ConnectionView (No indicators)") {
    ConnectionViewPreview(configuration: .normalNoIndicators).make()
}
