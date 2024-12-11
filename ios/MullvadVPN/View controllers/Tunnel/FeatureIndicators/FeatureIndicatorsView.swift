//
//  FeaturesIndicatorsView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-12-06.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct FeatureIndicatorsView<ViewModel>: View where ViewModel: ChipViewModelProtocol {
    @ObservedObject var viewModel: ViewModel

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Text(LocalizedStringKey("Active features"))
                .font(.footnote.weight(.semibold))
                .foregroundStyle(UIColor.primaryTextColor.color.opacity(0.6))

            ChipContainerView(viewModel: viewModel)
                .onTapGesture {
                    viewModel.isExpanded.toggle()
                }
        }
    }
}

#Preview("FeatureIndicatorsView") {
    FeatureIndicatorsView(viewModel: MockFeatureIndicatorsViewModel(isExpanded: true))
        .background(UIColor.secondaryColor.color)
}
