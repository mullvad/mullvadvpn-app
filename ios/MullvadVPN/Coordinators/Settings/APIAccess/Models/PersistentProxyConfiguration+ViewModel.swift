// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadTypes

extension PersistentProxyConfiguration {
    /// View model for socks configuration.
    var socksViewModel: AccessMethodViewModel.Socks {
        guard case let .socks5(config) = self else {
            return AccessMethodViewModel.Socks()
        }

        var socks = AccessMethodViewModel.Socks(
            server: "\(config.server)",
            port: "\(config.port)"
        )

        switch config.authentication {
        case let .authentication(userCredential):
            socks.username = userCredential.username
            socks.password = userCredential.password
            socks.authenticate = true

        case .noAuthentication:
            socks.authenticate = false
        }

        return socks
    }

    /// View model for shadowsocks configuration.
    var shadowsocksViewModel: AccessMethodViewModel.Shadowsocks {
        guard case let .shadowsocks(config) = self else {
            return AccessMethodViewModel.Shadowsocks()
        }
        return AccessMethodViewModel.Shadowsocks(
            server: "\(config.server)",
            port: "\(config.port)",
            password: config.password,
            cipher: config.cipher
        )
    }
}
