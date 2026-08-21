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

@testable import MullvadREST

public struct MockRelayCache: RelayCacheProtocol {
    public init() {}

    public func read() throws -> MullvadREST.CachedRelays {
        CachedRelays(
            relays: ServerRelaysResponseStubs.sampleRelays,
            updatedAt: Date()
        )
    }

    public func readPrebundledRelays() throws -> MullvadREST.CachedRelays {
        try self.read()
    }

    public func write(record: MullvadREST.StoredRelays) throws {}
}
