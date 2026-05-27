//
//  RelayFilterSettingsView.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2026-05-07.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

protocol RelayFilterSettingsViewModelProtocol {
    var filters: [SelectLocationFilter] { get }
    func onFilterTapped(_ filterr: SelectLocationFilter)
}

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
                    HStack {
                        Image.mullvadIconInfo.resizable().frame(width: 14, height: 14).opacity(0.6)
                        // Should this be "Filters are overridden when using the automatic location", which is an existing localised text already used elsewhere in the UI?
                        Text("Filters are overridden when using an automatic location").font(.mullvadMini)
                            .foregroundStyle(
                                Color.mullvadTextSecondary)
                        Spacer()
                    }
                    .padding(EdgeInsets(top: 4, leading: 16, bottom: 0, trailing: 0))
                }

            }
            .padding(EdgeInsets(top: 0, leading: 0, bottom: 4, trailing: 16))
            .background(Color.mullvadBackground)
        }
    }
}

private final class MockSettingsViewModel: RelayFilterSettingsViewModelProtocol, ObservableObject {
    var filters: [SelectLocationFilter]

    init(filters: [SelectLocationFilter]) {
        self.filters = filters
    }

    func onFilterTapped(_ filterr: SelectLocationFilter) {}
}

#Preview {
    RelayFilterSelection.SettingsView<MockSettingsViewModel>(
        viewModel: MockSettingsViewModel(
            filters: [.daita, .obfuscation]
        ))
}
