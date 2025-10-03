//
//  RecentConnectionRepositoryProtocol.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-09-24.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

public protocol RecentConnectionRepositoryProtocol: Actor {
    func add(_ recentConnection: RecentConnection, maxLimit: Int)
    func clear()
    func all() -> [RecentConnection]
}
