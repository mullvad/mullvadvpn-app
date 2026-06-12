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
                activeFilter: viewModel.featureFilters,
                labelStyle: .specific,
                automaticLocationIsActive: false,
                shouldShowAutomaticFilterOverrideNotice: viewModel.shouldShowAutomaticFilterOverrideNotice
            ) { filter in
                viewModel.onFeatureChipTapped?(filter)
            } onRemove: { _ in
            }
        }
    }
}

private final class MockSettingsViewModel: RelayFilterSettingsViewModelProtocol, ObservableObject {
    var featureFilters: [SelectLocationFilter]
    var shouldShowAutomaticFilterOverrideNotice: Bool
    var onFeatureChipTapped: ((SelectLocationFilter) -> Void)?

    init(filters: [SelectLocationFilter], automaticLocationIsActive: Bool) {
        self.featureFilters = filters
        self.shouldShowAutomaticFilterOverrideNotice = automaticLocationIsActive
    }
}

#Preview {
    RelayFilterSelection.SettingsView<MockSettingsViewModel>(
        viewModel: MockSettingsViewModel(
            filters: [.daita, .obfuscation(.quic)],
            automaticLocationIsActive: true
        )
    )
}
