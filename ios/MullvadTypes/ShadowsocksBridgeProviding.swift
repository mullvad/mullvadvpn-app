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

public protocol SwiftShadowsocksBridgeProviding: Sendable {
    func bridge() -> ShadowsocksConfiguration?
}

public final class SwiftShadowsocksBridgeProvider: SwiftShadowsocksBridgeProviding, Sendable {
    let provider: SwiftShadowsocksBridgeProviding

    public init(provider: SwiftShadowsocksBridgeProviding) {
        self.provider = provider
    }

    public func bridge() -> ShadowsocksConfiguration? {
        provider.bridge()
    }
}
