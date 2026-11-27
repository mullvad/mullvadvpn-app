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

struct ShadowsocksCipherPickerView: View {
    @State var viewModel: any ShadowsocksCipherPickerViewModelProtocol
    @Environment(\.dismiss) private var dismiss

    private let itemFactory = SegmentedListItemFactory()

    init(viewModel: any ShadowsocksCipherPickerViewModelProtocol) {
        _viewModel = State(initialValue: viewModel)
    }

    var body: some View {
        ScrollViewReader { proxy in
            ScrollView {
                VStack(alignment: .leading, spacing: 0) {
                    ForEach(Array(viewModel.ciphers.enumerated()), id: \.element) { index, item in
                        makeListItem(item, at: index)
                            .environment(\.isNestedInSegmentedListItem, index > 0)
                            .padding(.bottom, 1.0)
                    }
                }
                .padding(16.0)
            }
            .scrollBounceBehavior(.basedOnSize)
            .onAppear {
                proxy.scrollTo(viewModel.current)
            }
        }
        .accessibilityIdentifier(.accessMethodProtocolCipherPickerView)
        .navigationTitle("Cipher")
        .toolbarTitleDisplayMode(.inline)
        .background {
            Color.mullvadBackground
                .ignoresSafeArea()
        }
    }

    private func makeListItem(_ item: ShadowsocksCipherPickerItem, at index: Int) -> some View {
        SegmentedListItem(
            isLastInList: index == viewModel.ciphers.count - 1,
            accessibilityIdentifier: .cipherMethodType(item.cipher),
            accessibilityLabel: item.cipher,
            leading: {
                itemFactory.leading(
                    for: .generic(
                        title: item.text,
                        subtitle: item.detailText,
                        isSelected: item.text == viewModel.current
                    )
                )
            },
            onSelect: {
                viewModel.select(item)
                dismiss()
            }
        )
        .id(item.cipher)
    }
}
