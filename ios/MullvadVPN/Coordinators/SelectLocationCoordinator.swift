//
//  SelectLocationCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 29/01/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadTypes
import Routing
import UIKit

import MullvadSettings

class SelectLocationCoordinator: Coordinator, Presentable, Presenting, RelayCacheTrackerObserver {
    private let tunnelManager: TunnelManager
    private let relayCacheTracker: RelayCacheTracker
    private var cachedRelays: CachedRelays?

    let navigationController: UINavigationController

    var presentedViewController: UIViewController {
        navigationController
    }

    var selectLocationViewController: SelectLocationViewController? {
        return navigationController.viewControllers.first {
            $0 is SelectLocationViewController
        } as? SelectLocationViewController
    }

    var relayFilter: RelayFilter {
        switch tunnelManager.settings.relayConstraints.filter {
        case .any:
            return RelayFilter()
        case let .only(filter):
            return filter
        }
    }

    var didFinish: ((SelectLocationCoordinator, RelayLocation?) -> Void)?

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
        let selectLocationViewController = SelectLocationViewController()

        selectLocationViewController.didSelectRelay = { [weak self] relay in
            guard let self else { return }

            var relayConstraints = tunnelManager.settings.relayConstraints
            relayConstraints.locations = .only(RelayLocations(
                locations: [relay],
                customListId: nil
            ))

            tunnelManager.updateSettings([.relayConstraints(relayConstraints)]) {
                self.tunnelManager.startTunnel()
            }

            didFinish?(self, relay)
        }

        selectLocationViewController.navigateToFilter = { [weak self] in
            guard let self else { return }

            let coordinator = makeRelayFilterCoordinator(forModalPresentation: true)
            coordinator.start()

            presentChild(coordinator, animated: true)
        }

        selectLocationViewController.didUpdateFilter = { [weak self] filter in
            guard let self else { return }

            var relayConstraints = tunnelManager.settings.relayConstraints
            relayConstraints.filter = .only(filter)

            tunnelManager.updateSettings([.relayConstraints(relayConstraints)])
        }

        selectLocationViewController.didFinish = { [weak self] in
            guard let self else { return }

            didFinish?(self, nil)
        }

        relayCacheTracker.addObserver(self)

        if let cachedRelays = try? relayCacheTracker.getCachedRelays() {
            self.cachedRelays = cachedRelays
            selectLocationViewController.setCachedRelays(cachedRelays, filter: relayFilter)
        }

        selectLocationViewController.relayLocation =
            tunnelManager.settings.relayConstraints.locations.value?.locations.first

        navigationController.pushViewController(selectLocationViewController, animated: false)
    }

    private func makeRelayFilterCoordinator(forModalPresentation isModalPresentation: Bool)
        -> RelayFilterCoordinator {
        let navigationController = CustomNavigationController()

        let relayFilterCoordinator = RelayFilterCoordinator(
            navigationController: navigationController,
            tunnelManager: tunnelManager,
            relayCacheTracker: relayCacheTracker
        )

        relayFilterCoordinator.didFinish = { [weak self] coordinator, filter in
            if let cachedRelays = self?.cachedRelays, let filter {
                self?.selectLocationViewController?.setCachedRelays(cachedRelays, filter: filter)
            }

            coordinator.dismiss(animated: true)
        }

        return relayFilterCoordinator
    }

    func relayCacheTracker(
        _ tracker: RelayCacheTracker,
        didUpdateCachedRelays cachedRelays: CachedRelays
    ) {
        self.cachedRelays = cachedRelays

        selectLocationViewController?.setCachedRelays(cachedRelays, filter: relayFilter)
    }
}
