// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadSettings
import SwiftUI

struct ListCustomListsView: View {
    let viewModel: ListCustomListsViewModel
    private let itemFactory = SegmentedListItemFactory()

    @Environment(\.dismissSheet) private var dismiss

    var body: some View {
        VStack(spacing: 4) {
            ForEach(viewModel.customLists, id: \.id) { list in
                SegmentedListItem(
                    accessibilityIdentifier:
                        .editCustomListListItem(list.name),
                    leadingAndTrailingDestination: {
                        LazyView(
                            CustomListView(
                                viewModel: CustomListViewModelImpl(
                                    customList: list,
                                    interactor: viewModel.customListInteractor,
                                    context: viewModel.multihopContext
                                )
                            )
                        )
                    },
                    leading: {
                        itemFactory
                            .leading(
                                for: .generic(title: list.name)
                            )
                    },
                    trailing: {
                        itemFactory.trailing(
                            for: .drillDown(title: "", breadcrumb: nil)
                        )
                    },
                )
            }
        }
        .scrollable(fill: [.vertical, .horizontal], alignment: .top)
        .navigationTitle("Edit custom list")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            MullvadDoneToolbarItem { dismiss() }
        }
        .padding()
        .background(Color.mullvadBackground.ignoresSafeArea())
        .accessibilityIdentifier(.listCustomListsView)
    }
}

#Preview {
    NavigationStack {
        ListCustomListsView(
            viewModel: MockListCustomListsViewModel()
        )
    }
}
