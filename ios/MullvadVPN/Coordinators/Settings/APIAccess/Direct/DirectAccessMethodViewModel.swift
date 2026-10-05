// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Combine
import MullvadSettings
import MullvadTypes
import SwiftUI

@MainActor
protocol DirectAccessMethodViewModelProtocol {
    var isEnabled: Bool { get set }
}

@Observable
final class DirectAccessMethodViewModel: DirectAccessMethodViewModelProtocol {
    typealias OnChange = (PersistentAccessMethod) -> Void

    private var accessMethod: PersistentAccessMethod
    let onChange: OnChange

    var isEnabled: Bool {
        didSet {
            guard isEnabled != oldValue else { return }
            self.accessMethod.isEnabled = isEnabled
            onChange(self.accessMethod)
        }
    }

    init(accessMethod: PersistentAccessMethod, onChange: @escaping OnChange) {
        self.accessMethod = accessMethod
        self.onChange = onChange
        self.isEnabled = accessMethod.isEnabled
    }
}
