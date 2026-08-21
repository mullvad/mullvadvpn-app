// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
