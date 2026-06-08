//
//  MockRelayCacheProvider.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-06-08.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadREST

public struct MockRelayCacheTracker: RelayCacheTrackerProviding {
    public init() {}

    public func getCachedRelays() throws -> MullvadREST.CachedRelays {
        CachedRelays(
            relays: ServerRelaysResponseStubs.sampleRelays,
            updatedAt: Date()
        )
    }
}
