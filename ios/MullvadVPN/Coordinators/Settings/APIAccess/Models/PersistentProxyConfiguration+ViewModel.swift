//
//  PersistentProxyConfiguration+ViewModel.swift
//  MullvadVPN
//
//  Created by pronebird on 29/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

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
