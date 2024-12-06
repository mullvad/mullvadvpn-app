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
            Color(uiColor: .secondaryColor)
                .ignoresSafeArea()

            VStack(spacing: UIMetrics.padding16) {
                HStack(alignment: .top) {
                    Text(LocalizedStringKey("Active features"))
                        .multilineTextAlignment(.leading)
                        .font(.body)
                        .foregroundStyle(Color(uiColor: .secondaryTextColor))
                        .padding(.leading, UIMetrics.padding8)
                    Spacer()
                }

                ScrollView {
                    HStack {
                        ChipContainerView(viewModel: viewModel)
                        Text(LocalizedStringKey("Active features"))
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

private class FeaturesIndicatoresMockViewModel: ChipViewModelProtocol {
    @Published var chips: [ChipModel] = [
        ChipModel(name: LocalizedStringKey("DAITA")),
        ChipModel(name: LocalizedStringKey("Obfuscation")),
        ChipModel(name: LocalizedStringKey("Quantum resistance")),
        ChipModel(name: LocalizedStringKey("Multihop")),
    ]
}
