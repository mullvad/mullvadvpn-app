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
            ActiveFilterView(
                activeFilter: viewModel.filters,
                labelStyle: .specific,
                automaticLocationIsActive: false
            ) { filter in
                viewModel.onFilterTapped(filter)
            } onRemove: { _ in
            }
            .padding(EdgeInsets(top: 0, leading: 0, bottom: 4, trailing: 16))
            .background(Color.mullvadBackground)
        }
    }
}

private final class MockSettingsViewModel: RelayFilterSettingsViewModelProtocol, ObservableObject {
    var filters: [SelectLocationFilter]
    var shouldShowAutomaticFilterOverrideNotice: Bool

    init(filters: [SelectLocationFilter], automaticLocationIsActive: Bool) {
        self.filters = filters
        self.shouldShowAutomaticFilterOverrideNotice = automaticLocationIsActive
    }

    func onFilterTapped(_ filterr: SelectLocationFilter) {}
}

#Preview {
    RelayFilterSelection.SettingsView<MockSettingsViewModel>(
        viewModel: MockSettingsViewModel(
            filters: [.daita, .obfuscation(.quic)],
            automaticLocationIsActive: true
        ))
}
