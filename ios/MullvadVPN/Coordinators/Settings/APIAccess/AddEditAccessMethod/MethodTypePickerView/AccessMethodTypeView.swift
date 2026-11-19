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

struct AccessMethodTypeView: View {
    @State private var viewModel: any AccessMethodTypeViewModelProtocol
    @Environment(\.dismiss) private var dismiss

    private let itemFactory = SegmentedListItemFactory()

    init(viewModel: any AccessMethodTypeViewModelProtocol) {
        _viewModel = State(initialValue: viewModel)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            ForEach(Array(viewModel.list.enumerated()), id: \.element) { index, item in
                makeListItem(item, at: index)
                    .environment(\.isNestedInSegmentedListItem, index > 0)
                    .padding(.bottom, 1.0)
            }
        }
        .padding(16.0)
        .scrollable()
        .accessibilityIdentifier(.accessMethodProtocolPickerView)
        .navigationTitle("Type")
        .toolbarTitleDisplayMode(.inline)
        .background {
            Color.mullvadBackground
                .ignoresSafeArea()
        }
    }

    private func makeListItem(_ item: AccessMethodKind, at index: Int) -> some View {
        SegmentedListItem(
            isLastInList: index == viewModel.list.count - 1,
            accessibilityIdentifier: .accessMethodType(item.localizedDescription),
            leading: {
                itemFactory.leading(
                    for: .generic(
                        title: item.localizedDescription,
                        isSelected: item == viewModel.current
                    )
                )
            },
            onSelect: {
                viewModel.select(item)
                dismiss()
            }
        )
    }
}
