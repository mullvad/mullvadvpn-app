//
//  RecentsConnectionDataSource.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-09-30.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes

final class RecentsConnectionDataSource: LocationDataSourceProtocol {
    typealias Node = RecentConnectionLocationNode
    private(set) var nodes = [Node]()
    let repository: RecentConnectionRepositoryProtocol
    let settings: LatestTunnelSettings
    let userSelectedLocationFinder: UserSelectedLocationFinder

    init(
        repository: RecentConnectionRepositoryProtocol,
        settings: LatestTunnelSettings,
        userSelectedLocationFinder: UserSelectedLocationFinder
    ) {
        self.repository = repository
        self.settings = settings
        self.userSelectedLocationFinder = userSelectedLocationFinder
    }

    @MainActor
    func reload(allLocationNodes: [LocationNode]) {
        Task { [weak self] in
            guard let self else { return }
            let recents = await repository.all()
            self.nodes = recents.compactMap {
                RecentConnectionLocationNodeBuilder(
                    userSelectedLocationFinder: self.userSelectedLocationFinder,
                    settings: settings,
                    recentConnection: $0
                ).node
            }
        }
    }

    func search(by text: String) -> [Node] {
        guard !text.isEmpty else {
            return nodes
        }
        return nodes.filter { $0.name.fuzzyMatch(text) }
    }
}
