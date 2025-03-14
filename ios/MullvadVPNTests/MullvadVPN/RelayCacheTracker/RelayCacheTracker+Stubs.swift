//
//  RelayCacheTracker+Stubs.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-10-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

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

    func getCachedRelays() throws -> CachedRelays {
        CachedRelays(relays: ServerRelaysResponseStubs.sampleRelays, updatedAt: Date())
    }

    func getNextUpdateDate() -> Date {
        Date()
    }

    func addObserver(_ observer: RelayCacheTrackerObserver) {}

    func removeObserver(_ observer: RelayCacheTrackerObserver) {}

    func refreshCachedRelays() throws {}
}
