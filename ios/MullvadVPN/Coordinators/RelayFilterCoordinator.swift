//
//  RelayFilterCoordinator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-06-14.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadTypes
import Routing
import UIKit

class RelayFilterCoordinator: Coordinator, Presentable, RelayCacheTrackerObserver {
    private let tunnelManager: TunnelManager
    private let relayCacheTracker: RelayCacheTracker
    private var cachedRelays: CachedRelays?

    let navigationController: UINavigationController

    var presentedViewController: UIViewController {
        return navigationController
    }

    var relayFilterViewController: RelayFilterViewController? {
        return navigationController.viewControllers.first {
            $0 is RelayFilterViewController
        } as? RelayFilterViewController
    }

    var relayFilter: RelayFilter {
        switch tunnelManager.settings.relayConstraints.filter {
        case .any:
            return RelayFilter()
        case let .only(filter):
            return filter
        }
    }

    var didFinish: ((RelayFilterCoordinator, RelayFilter?) -> Void)?

    init(
        navigationController: UINavigationController,
        tunnelManager: TunnelManager,
        relayCacheTracker: RelayCacheTracker
    ) {
        self.navigationController = navigationController
        self.tunnelManager = tunnelManager
        self.relayCacheTracker = relayCacheTracker
    }

    func start() {
        let relayFilterViewController = RelayFilterViewController()

        relayFilterViewController.onApplyFilter = { [weak self] filter in
            guard let self else { return }

            var relayConstraints = tunnelManager.settings.relayConstraints
            relayConstraints.filter = .only(filter)

            tunnelManager.setRelayConstraints(relayConstraints)

            didFinish?(self, filter)
        }

        relayFilterViewController.didFinish = { [weak self] in
            guard let self else { return }

            didFinish?(self, nil)
        }

        relayCacheTracker.addObserver(self)

        if let cachedRelays = try? relayCacheTracker.getCachedRelays() {
            self.cachedRelays = cachedRelays
            relayFilterViewController.setCachedRelays(cachedRelays, filter: relayFilter)
        }

        navigationController.pushViewController(relayFilterViewController, animated: false)
    }

    func relayCacheTracker(
        _ tracker: RelayCacheTracker,
        didUpdateCachedRelays cachedRelays: CachedRelays
    ) {
        self.cachedRelays = cachedRelays
        relayFilterViewController?.setCachedRelays(cachedRelays, filter: relayFilter)
    }
}
