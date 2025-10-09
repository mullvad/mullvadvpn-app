//
//  RecentConnectionRepositoryProtocol.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-09-24.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

public protocol RecentConnectionRepositoryProtocol {
    func add(_ recentConnection: RecentConnection)
    func clear()
    func all() -> [RecentConnection]
}
