//
//  FeaturesIndicatoresView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-12-06.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI
struct FeaturesIndicatoresView<ViewModel>: View where ViewModel: ChipViewModelProtocol {
    @ObservedObject var viewModel: ViewModel

    var body: some View {
        ZStack {
            VStack(spacing: UIMetrics.padding16) {
                HStack(alignment: .top) {
                    Text(LocalizedStringKey("Active features"))
                        .lineLimit(1)
                        .font(.body.weight(.semibold))
                        .foregroundStyle(.white.opacity(0.6))
                        .padding(.leading, UIMetrics.padding8)
                    Spacer()
                }

                ScrollView {
                    HStack {
                        ChipContainerView(viewModel: viewModel)
                    }
                }
                Spacer()
            }
        }
    }
}

#Preview("FeaturesIndicatoresView") {
    FeaturesIndicatoresView(viewModel: FeaturesIndicatoresMockViewModel())
}
