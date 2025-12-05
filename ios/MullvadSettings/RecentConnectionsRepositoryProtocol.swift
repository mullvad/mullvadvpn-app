//
//  RecentConnectionsRepositoryProtocol.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-10-15.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadTypes

public enum RecentConnectionsResult {
    case success(RecentConnections)
    case failure(Error)
}
public protocol RecentConnectionsRepositoryProtocol {
    var recentConnectionsPublisher: AnyPublisher<RecentConnectionsResult, Never> { get }
    func disable()
    func enable(_ selectedEntryRelays: UserSelectedRelays?, selectedExitRelays: UserSelectedRelays)
    func add(_ selectedEntryRelays: UserSelectedRelays?, selectedExitRelays: UserSelectedRelays)
    func initiate()
}
