//
//  RecentConnectionRepositoryProtocol.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-09-24.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

public protocol RecentConnectionRepositoryProtocol {
    func add(_ recentConnection: RecentConnection) throws
    func clear() throws
    func all() throws -> [RecentConnection]
}
