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
        VStack {
            HStack(alignment: .top) {
                Text(LocalizedStringKey("Active features"))
                    .font(.footnote.weight(.semibold))
                    .foregroundStyle(.white.opacity(0.6))
                Spacer()
            }

            ChipContainerView(viewModel: viewModel)
        }
    }
}

#Preview("FeaturesIndicatoresView") {
    FeatureIndicatorsView(viewModel: FeaturesIndicatoresMockViewModel())
        .background(UIColor.secondaryColor.color)
}
