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
    var current: Binding<AccessMethodKind> { get }
    var list: [AccessMethodKind] { get }
}

@Observable
class AccessMethodTypeViewModel: AccessMethodTypeViewModelProtocol {
    let current: Binding<AccessMethodKind>
    let list: [AccessMethodKind]

    init(current: Binding<AccessMethodKind>, list: [AccessMethodKind]) {
        self.current = current
        self.list = list
    }
}
