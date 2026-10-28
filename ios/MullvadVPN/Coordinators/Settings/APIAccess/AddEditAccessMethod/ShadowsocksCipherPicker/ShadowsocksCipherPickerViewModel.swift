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
    var list: [String] { get }
}

@Observable
class ShadowsocksCipherPickerViewModel: ShadowsocksCipherPickerViewModelProtocol {
    let current: Binding<String>
    let list: [String]

    init(current: Binding<String>, list: [String]) {
        self.current = current
        self.list = list
    }
}
