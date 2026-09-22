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

public class ShadowsocksCacheCleaner: MullvadAccessMethodChangeListening, @unchecked Sendable {
    let cache: ShadowsocksConfigurationCacheProtocol
    let lock = NSLock()
    var lastChangedUUID = UUID(uuidString: "00000000-0000-0000-0000-000000000000")!

    public init(cache: ShadowsocksConfigurationCacheProtocol) {
        self.cache = cache
    }

    public func accessMethodChangedTo(_ uuid: UUID) {
        lock.withLock {
            if lastChangedUUID == AccessMethodRepository.bridgeId {
                try? cache.clear()
            }
            lastChangedUUID = uuid
        }
    }
}
