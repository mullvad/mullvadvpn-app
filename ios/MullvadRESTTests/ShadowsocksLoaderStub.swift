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
import MullvadSettings
import MullvadTypes

@testable import MullvadREST

struct ShadowsocksLoaderStub: ShadowsocksLoaderProtocol, SwiftShadowsocksBridgeProviding {
    func bridge() -> ShadowsocksConfiguration? {
        try? load()
    }

    var configuration: ShadowsocksConfiguration
    var error: Error?

    func clear() throws {
        try load()
    }

    @discardableResult
    func load() throws -> ShadowsocksConfiguration {
        if let error { throw error }
        return configuration
    }
}
