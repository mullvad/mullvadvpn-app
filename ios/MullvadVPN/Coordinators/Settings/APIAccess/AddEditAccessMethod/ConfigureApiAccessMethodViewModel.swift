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
protocol ConfigureApiAccessMethodViewModelProtocol: AnyObject, Observable {
    var title: LocalizedStringKey { get }
    var saveButtonTitle: LocalizedStringKey { get }
    var dismissButtonTitle: LocalizedStringKey { get }
    var dismissButtonImage: Image? { get }
    var name: String { get set }
    var type: AccessMethodKind { get set }
    var server: String { get set }
    var port: String { get set }
}

@Observable
class ConfigureApiAccessMethodViewModel: ConfigureApiAccessMethodViewModelProtocol {
    var accessMethod: PersistentAccessMethod?
    var name: String
    var server: String
    var port: String
    var type: AccessMethodKind

    init(accessMethod: PersistentAccessMethod? = nil) {
        self.accessMethod = accessMethod
        self.name = accessMethod?.name ?? ""
        self.type = accessMethod?.kind ?? .shadowsocks
        switch accessMethod?.proxyConfiguration {
        case .shadowsocks(let configuration):
            server = configuration.server.debugDescription
            port = "\(configuration.port)"

        case .socks5(let configuration):
            server = configuration.server.debugDescription
            port = "\(configuration.port)"

        default:
            server = ""
            port = ""
        }
    }

    var title: LocalizedStringKey {
        accessMethod == nil ? "Add method" : "Edit method"
    }

    var dismissButtonTitle: LocalizedStringKey {
        accessMethod == nil ? "Cancel" : "Back"
    }

    var dismissButtonImage: Image? {
        accessMethod == nil ? nil : Image(.iconBack)
    }

    var saveButtonTitle: LocalizedStringKey {
        accessMethod == nil ? "Add" : "Save"
    }
}
