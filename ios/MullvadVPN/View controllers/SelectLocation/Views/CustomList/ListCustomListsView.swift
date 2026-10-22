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
    let customLists: [CustomListLocationNode]
    let availableLocations: [LocationNode]
    let viewModel: SelectLocationViewModel
    private let itemFactory = SegmentedListItemFactory()

    @Environment(\.dismissSheet) private var dismiss
    var body: some View {
        VStack(spacing: 4) {
            ForEach(customLists, id: \.id) { list in
                SegmentedListItem(
                    accessibilityIdentifier:
                        .editCustomListListItem(list.name),
                    leadingAndTrailingDestination: {
                        AddCustomListView(
                            availableLocations: availableLocations,
                            customList: list,
                            viewModel: viewModel
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
            customLists: [
                .init(
                    name: "My Custom list",
                    code: "my-list",
                    customList: .init(name: "", locations: [])
                ),
                .init(
                    name: "My Custom list 2",
                    code: "my-list",
                    customList: .init(name: "", locations: [])
                ),
            ],
            availableLocations: [],
            viewModel: MockSelectLocationViewModel()
        )
    }
}
