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

protocol AccessMethodTypeViewModelProtocol {
    var current: AccessMethodKind { get }
    var list: [AccessMethodKind] { get }
    func select(_ item: AccessMethodKind)
}

@Observable
class AccessMethodTypeViewModel: AccessMethodTypeViewModelProtocol {
    private(set) var current: AccessMethodKind

    let list: [AccessMethodKind]
    var currentBinding: Binding<AccessMethodKind>

    init(current: Binding<AccessMethodKind>, list: [AccessMethodKind]) {
        self.current = current.wrappedValue
        self.currentBinding = current
        self.list = list
    }

    func select(_ item: AccessMethodKind) {
        currentBinding.wrappedValue = item
        current = currentBinding.wrappedValue
    }
}
