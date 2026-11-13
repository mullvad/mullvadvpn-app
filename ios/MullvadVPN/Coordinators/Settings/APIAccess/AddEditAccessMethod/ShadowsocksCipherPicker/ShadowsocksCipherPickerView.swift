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
    private let currentCipher: String

    init(viewModel: any ShadowsocksCipherPickerViewModelProtocol) {
        _viewModel = State(initialValue: viewModel)
        currentCipher = viewModel.current.wrappedValue
    }

    var body: some View {
        SingleChoiceList(
            options: viewModel.ciphers.map(\.cipher),
            value: viewModel.current,
            tableAccessibilityIdentifier: AccessibilityIdentifier.accessMethodProtocolCipherPickerView.asString,
            itemDescription: { value in
                guard let item = viewModel.ciphers.first(where: { $0.cipher == value }) else {
                    return ""
                }
                let description = item.detailText != nil ? "(\(item.detailText!))" : ""
                return "\(item.cipher)\(description)"
            },
            isEnabled: { value in
                viewModel.ciphers.first(where: { $0.cipher == value })?.isEnabled ?? true
            }
        )
        .onChange(of: currentCipher) { oldValue, newValue in
            dismiss()
        }
        .navigationTitle("Cipher")
        .toolbarTitleDisplayMode(.inline)
    }
}
