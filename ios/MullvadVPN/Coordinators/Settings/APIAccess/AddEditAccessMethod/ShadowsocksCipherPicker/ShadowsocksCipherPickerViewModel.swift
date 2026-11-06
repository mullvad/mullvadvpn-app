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
    var current: Binding<String> { get }
    var ciphers: [ShadowsocksCipherPickerItem] { get }
}

struct ShadowsocksCipherPickerItem: Equatable {
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
    let current: Binding<String>
    let ciphers: [ShadowsocksCipherPickerItem]

    init(ciphers: [String], current: Binding<String>) {
        self.current = current

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
}
