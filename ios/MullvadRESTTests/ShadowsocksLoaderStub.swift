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
import MullvadRustRuntime
import MullvadSettings
import MullvadTypes

@testable import MullvadREST

final class ShadowsocksLoaderStub: ShadowsocksLoaderProtocol, ShadowsocksBridgeProvider {
    func bridge() -> ShadowsocksWrapper? {
        guard let shadowsocks = try? load() else {
            return nil
        }
        return newShadowsocksAccessMethodSetting(
            address: shadowsocks.address.rawValue,
            port: shadowsocks.port,
            password: shadowsocks.password,
            cipher: shadowsocks.cipher)
    }

    let configuration: ShadowsocksConfiguration
    let error: Error?

    init(configuration: ShadowsocksConfiguration, error: Error? = nil) {
        self.configuration = configuration
        self.error = error
    }

    func clear() throws {
        try load()
    }

    @discardableResult
    func load() throws -> ShadowsocksConfiguration {
        if let error { throw error }
        return configuration
    }
}
