//
//  RelayFilterSettingsView.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2026-05-07.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

extension RelayFilterSelection {
    struct SettingsView<SettingsViewModel: RelayFilterSettingsViewModelProtocol & ObservableObject>: View {
        @ObservedObject var viewModel: SettingsViewModel

        var body: some View {
            VStack {
                if !viewModel.filters.isEmpty {
                    ActiveFilterView(
                        activeFilter: viewModel.filters,
                        automaticLocationIsActive: false
                    ) { filter in
                        viewModel.onFilterTapped(filter)
                    } onRemove: { _ in
                    }
                    if viewModel.automaticLocationIsActive {
                        HStack {
                            Image.mullvadIconInfo.resizable().frame(width: 14, height: 14).opacity(0.6)
                            Text("Filters are overridden when using an automatic location").font(.mullvadMini)
                                .foregroundStyle(
                                    Color.mullvadTextSecondary)
                            Spacer()
                        }
                        .padding(EdgeInsets(top: 4, leading: 16, bottom: 0, trailing: 0))
                    }
                }

            }
            .padding(EdgeInsets(top: 0, leading: 0, bottom: 4, trailing: 16))
            .background(Color.mullvadBackground)
        }
    }
}

private final class MockSettingsViewModel: RelayFilterSettingsViewModelProtocol, ObservableObject {
    var filters: [SelectLocationFilter]
    var automaticLocationIsActive: Bool

    init(filters: [SelectLocationFilter], automaticLocationIsActive: Bool) {
        self.filters = filters
        self.automaticLocationIsActive = automaticLocationIsActive
    }

    func onFilterTapped(_ filterr: SelectLocationFilter) {}
}

#Preview {
    RelayFilterSelection.SettingsView<MockSettingsViewModel>(
        viewModel: MockSettingsViewModel(
            filters: [.daita, .obfuscation],
            automaticLocationIsActive: true
        ))
}
