//
//  LocationCoordinator.swift
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

class LocationCoordinator: Coordinator, Presentable, Presenting, RelayCacheTrackerObserver {
    private let tunnelManager: TunnelManager
    private let relayCacheTracker: RelayCacheTracker
    private var cachedRelays: CachedRelays?

    let navigationController: UINavigationController

    var presentedViewController: UIViewController {
        navigationController
    }

    var selectLocationViewController: LocationViewController? {
        return navigationController.viewControllers.first {
            $0 is LocationViewController
        } as? LocationViewController
    }

    var relayFilter: RelayFilter {
        switch tunnelManager.settings.relayConstraints.filter {
        case .any:
            return RelayFilter()
        case let .only(filter):
            return filter
        }
    }

    var didFinish: ((LocationCoordinator, [RelayLocation]) -> Void)?

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
        let selectLocationViewController = LocationViewController()

        selectLocationViewController.didSelectRelays = { [weak self] locations, customListId in
            guard let self else { return }

            var relayConstraints = tunnelManager.settings.relayConstraints
            relayConstraints.locations = .only(RelayLocations(
                locations: locations,
                customListId: customListId
            ))

            tunnelManager.updateSettings([.relayConstraints(relayConstraints)]) {
                self.tunnelManager.startTunnel()
            }

            didFinish?(self, locations)
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

            didFinish?(self, [])
        }

        relayCacheTracker.addObserver(self)

        if let cachedRelays = try? relayCacheTracker.getCachedRelays() {
            self.cachedRelays = cachedRelays
            selectLocationViewController.setCachedRelays(cachedRelays, filter: relayFilter)
        }

        selectLocationViewController.relayLocations = tunnelManager.settings.relayConstraints.locations.value

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
