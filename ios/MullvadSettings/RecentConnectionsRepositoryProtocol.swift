//
//  RecentConnectionsRepositoryProtocol.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-10-15.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import MullvadTypes

public enum RecentLocationType: CaseIterable {
    case entry, exit
}
public protocol RecentConnectionsRepositoryProtocol {
    var isRecentConnectionsShown: Bool { get throws }
    func setRecentsEnabled(_ isEnabled: Bool) throws
    func add(_ location: UserSelectedRelays, to: RecentLocationType) throws
    func all() throws -> RecentConnections
}
