//
//  SettingsView.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2026-05-07.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

extension RelayFilterSelection {
    struct SettingsView<SettingsViewModel: ChipViewModelProtocol>: View {
        @ObservedObject var viewModel: SettingsViewModel

        var body: some View {
            VStack {
                ChipContainerView(viewModel: viewModel, style: .dialog, isExpanded: Binding.constant(true))
                HStack {
                    Image.mullvadIconInfo.resizable().frame(width: 14, height: 14).opacity(0.6)
                    // Should this be "Filters are overridden when using the automatic location", which is an existing localised text already used elsewhere in the UI?
                    Text("Filters are overridden when using an automatic location").font(.mullvadMini).foregroundStyle(
                        Color.mullvadTextSecondary)
                    Spacer()
                }
            }
            .padding(EdgeInsets(top: 0, leading: 16, bottom: 16, trailing: 16))  // TODO
            .background(Color.mullvadBackground)
        }
    }
}

#Preview {
    RelayFilterSelection.SettingsView(viewModel: MockFeatureIndicatorsViewModel(maxChips: 2))
}
