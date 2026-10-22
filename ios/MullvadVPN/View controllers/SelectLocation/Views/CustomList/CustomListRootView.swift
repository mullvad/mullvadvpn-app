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

struct CustomListRootView: View {
    var viewModel: SelectLocationViewModel
    @Environment(\.dismiss) private var dismiss
    let type: CustomListSheetType
    var body: some View {
        NavigationStack {
            switch type {
            case .add:
                AddCustomListView(
                    availableLocations: viewModel.currentLocationContext.customListAvailableLocations,
                    customList: nil,
                    viewModel: viewModel,
                    isRootView: true
                )
            case .list:
                ListCustomListsView(
                    customLists: viewModel.currentLocationContext.customLists
                        .compactMap { $0.asCustomListNode },
                    availableLocations: viewModel.currentLocationContext.customListAvailableLocations,
                    viewModel: viewModel
                )
            case .edit(let customListNode):
                AddCustomListView(
                    availableLocations: viewModel.currentLocationContext.customListAvailableLocations,
                    customList: customListNode,
                    viewModel: viewModel,
                    isRootView: true
                )
            }
        }
        .environment(\.dismissSheet, { dismiss() })
    }
}

private struct DismissSheetKey: EnvironmentKey {
    static let defaultValue: @MainActor @Sendable () -> Void = {}
}

extension EnvironmentValues {
    var dismissSheet: @MainActor @Sendable () -> Void {
        get { self[DismissSheetKey.self] }
        set { self[DismissSheetKey.self] = newValue }
    }
}
