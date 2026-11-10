// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadTypes
import SwiftUI

struct CustomListLocationSelectionView<ViewModel: CustomListViewModel>: View {
    @Bindable var viewModel: ViewModel

    var body: some View {
        ScrollView {
            LazyVStack(spacing: 0) {
                LocationsListView(
                    locations: $viewModel.availableLocations.children,
                    multihopContext: .entry,  // This is not used in this view
                    onSelectLocation: {
                        ($0.asCustomListSelectableLocationNode)?.isCustomListSelected.toggle()
                    },
                    contextMenu: { _ in EmptyView() }
                )
            }
            .padding()
        }
        .background(Color.mullvadBackground.ignoresSafeArea())
        .accessibilityIdentifier(.editCustomListEditLocationsView)
    }
}

#Preview {
    @Previewable @State var viewModel = CustomListViewModelImpl(
        customList: nil,
        interactor: MockCustomListInteractor(),
        context: .exit,
        onDidSave: {}
    )
    CustomListLocationSelectionView(
        viewModel: viewModel
    )
}
