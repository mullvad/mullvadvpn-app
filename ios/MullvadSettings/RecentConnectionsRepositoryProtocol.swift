//
//  RecentConnectionsRepositoryProtocol.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-10-15.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
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
    func enable(
        _ selectedEntryConstraint: RelayConstraint<UserSelectedRelays>?,
        selectedExitConstraint: RelayConstraint<UserSelectedRelays>)
    func add(
        _ selectedEntryConstraint: RelayConstraint<UserSelectedRelays>,
        selectedExitConstraint: RelayConstraint<UserSelectedRelays>)
    func deleteCustomList(_ id: UUID)
    func load()
}
