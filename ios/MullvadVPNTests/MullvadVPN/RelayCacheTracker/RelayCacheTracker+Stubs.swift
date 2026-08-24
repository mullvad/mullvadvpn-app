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
import MullvadMockData

@testable import MullvadREST
@testable import MullvadTypes

struct RelayCacheTrackerStub: RelayCacheTrackerProtocol {
    func startPeriodicUpdates() {}

    func stopPeriodicUpdates() {}

    func updateRelays(completionHandler: ((sending Result<RelaysFetchResult, Error>) -> Void)?) -> Cancellable {
        AnyCancellable()
    }

    func fetchRelays(completionHandler: ((sending Result<RelaysFetchResult, Error>) -> Void)?) -> Cancellable {
        AnyCancellable()
    }

    func getCachedRelays() throws -> CachedRelays {
        CachedRelays(relays: ServerRelaysResponseStubs.sampleRelays, updatedAt: Date())
    }

    func getNextUpdateDate() -> Date {
        Date()
    }

    func refreshCachedRelays() throws {}
}
