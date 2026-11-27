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

protocol ShadowsocksCipherPickerViewModelProtocol {
    var current: String { get }
    var ciphers: [ShadowsocksCipherPickerItem] { get }
    func select(_ cipher: ShadowsocksCipherPickerItem)
}

struct ShadowsocksCipherPickerItem: Equatable, Hashable {
    let cipher: String
    let isEnabled: Bool

    var id: String { cipher }
    var text: String { cipher }
    var detailText: String? {
        isEnabled ? nil : NSLocalizedString("No longer supported", comment: "")
    }
}

@Observable
class ShadowsocksCipherPickerViewModel: ShadowsocksCipherPickerViewModelProtocol {
    let currentBinding: Binding<String>
    let ciphers: [ShadowsocksCipherPickerItem]

    private(set) var current: String

    init(ciphers: [String], current: Binding<String>) {
        self.currentBinding = current
        self.current = current.wrappedValue

        var items = ciphers.map { ShadowsocksCipherPickerItem(cipher: $0, isEnabled: true) }

        let selectedItem = ShadowsocksCipherPickerItem(
            cipher: current.wrappedValue,
            isEnabled: ciphers.contains(current.wrappedValue))

        if !items.contains(selectedItem) {
            items.append(selectedItem)
            items.sort { $0.id < $1.id }
        }

        self.ciphers = items
    }

    func select(_ item: ShadowsocksCipherPickerItem) {
        currentBinding.wrappedValue = item.cipher
        current = currentBinding.wrappedValue
    }
}
