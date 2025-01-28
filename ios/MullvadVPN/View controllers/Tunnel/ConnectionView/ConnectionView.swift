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

    var body: some View {
        Spacer()
            .accessibilityIdentifier(AccessibilityIdentifier.connectionView.asString)

        VStack(alignment: .leading, spacing: 0) {
            HeaderView(viewModel: connectionViewModel, isExpanded: $isExpanded)
                .padding(.bottom, headerViewBottomPadding)

            DetailsContainer(
                connectionViewModel: connectionViewModel,
                indicatorsViewModel: indicatorsViewModel,
                isExpanded: $isExpanded
            )
            .showIf(connectionViewModel.showsConnectionDetails)

            ButtonPanel(viewModel: connectionViewModel, action: action)
                .padding(.top, 16)
        }
        .padding(16)
        .background(BlurView(style: .dark))
        .cornerRadius(12)
        .padding(EdgeInsets(top: 16, leading: 16, bottom: 24, trailing: 16))
    }
}

extension ConnectionView {
    var headerViewBottomPadding: CGFloat {
        let hasIndicators = !indicatorsViewModel.chips.isEmpty
        let showConnectionDetails = connectionViewModel.showsConnectionDetails

        return isExpanded
            ? showConnectionDetails ? 16 : 0
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
