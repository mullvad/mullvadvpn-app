//
//  FeaturesIndicatoresView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-12-06.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI
struct FeatureIndicatorsView<ViewModel>: View where ViewModel: ChipViewModelProtocol {
    @ObservedObject var viewModel: ViewModel

    var body: some View {
        VStack(alignment: .leading) {
            Text(LocalizedStringKey("Active features"))
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.white.opacity(0.6))

            ChipContainerView(viewModel: viewModel)
                .onTapGesture {
                    viewModel.isExpanded.toggle()
                }
        }
    }
}

#Preview("FeaturesIndicatoresView") {
    FeatureIndicatorsView(viewModel: FeaturesIndicatoresMockViewModel())
        .background(UIColor.secondaryColor.color)
}
