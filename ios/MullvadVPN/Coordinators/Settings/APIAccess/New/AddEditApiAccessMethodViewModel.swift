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
import MullvadTypes
import SwiftUI

@MainActor
protocol AddEditApiAccessMethodViewModelProtocol {
    var title: LocalizedStringKey { get }
    var dismissButtonTitle: LocalizedStringKey { get }
    var name: String { get set }

    //    var type : PersistentProxyConfiguration { get set}
    //    var server : String { get set }
}

@Observable
class AddEditApiAccessMethodViewModel: AddEditApiAccessMethodViewModelProtocol {
    var name: String = ""

    let accessMethod: PersistentAccessMethod?

    var title: LocalizedStringKey {
        accessMethod == nil ? "Add method" : "Edit method"
    }

    var dismissButtonTitle: LocalizedStringKey {
        accessMethod == nil ? "Cancel" : "\(accessMethod!.name)"
    }

    init(accessMethod: PersistentAccessMethod? = nil) {
        self.accessMethod = accessMethod
    }

    //    var type: PersistentProxyConfiguration {
    //        accessMethod?.kind ?? .shadowsocks
    //    }
    //
    //    var server: String {
    //        accessMethod?.
    //    }

}
