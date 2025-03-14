//
//  MockRelayCache.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-03-10.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import Foundation
@testable import MullvadREST

public struct MockRelayCache: RelayCacheProtocol {
    public init() {}

    public func read() throws -> MullvadREST.StoredRelays {
        try .init(
            cachedRelays: CachedRelays(
                relays: ServerRelaysResponseStubs.sampleRelays,
                updatedAt: Date()
            )
        )
    }

    public func readPrebundledRelays() throws -> MullvadREST.StoredRelays {
        try self.read()
    }

    public func write(record: MullvadREST.StoredRelays) throws {}
}
