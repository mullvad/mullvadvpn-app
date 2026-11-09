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
    let type: CustomListSheetType

    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            switch type {
            case .add:
                CustomListView(
                    viewModel: CustomListViewModelImpl(
                        customList: nil,
                        interactor: viewModel.customListInteractor,
                        context: viewModel.multihopContext
                    ),
                    isRootView: true
                )
            case .list:
                ListCustomListsView(
                    viewModel: ListCustomListsViewModelImpl(
                        customListInteractor: viewModel.customListInteractor,
                        multihopContext: viewModel.multihopContext
                    )
                )
            case .edit(let customList):
                CustomListView(
                    viewModel: CustomListViewModelImpl(
                        customList: customList,
                        interactor: viewModel.customListInteractor,
                        context: viewModel.multihopContext
                    ),
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
