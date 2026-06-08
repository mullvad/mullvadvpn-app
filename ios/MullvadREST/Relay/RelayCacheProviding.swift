//
//  RelayCacheProviding.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-06-08.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

public protocol RelayCacheTrackerProviding: Sendable {
    func getCachedRelays() throws -> CachedRelays
}
