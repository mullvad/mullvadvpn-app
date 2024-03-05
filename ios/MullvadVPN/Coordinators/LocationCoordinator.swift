//
//  LocationCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 29/01/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadSettings
import MullvadTypes
import Routing
import UIKit

class LocationCoordinator: Coordinator, Presentable, Presenting, RelayCacheTrackerObserver {
    private let tunnelManager: TunnelManager
    private let relayCacheTracker: RelayCacheTracker
    private var cachedRelays: CachedRelays?
    private var customListRepository: CustomListRepositoryProtocol

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

    var didFinish: ((LocationCoordinator) -> Void)?

    init(
        navigationController: UINavigationController,
        tunnelManager: TunnelManager,
        relayCacheTracker: RelayCacheTracker,
        customListRepository: CustomListRepositoryProtocol
    ) {
        self.navigationController = navigationController
        self.tunnelManager = tunnelManager
        self.relayCacheTracker = relayCacheTracker
        self.customListRepository = customListRepository
    }

    func start() {
        let locationViewController = LocationViewController(customListRepository: customListRepository)
        locationViewController.delegate = self

        locationViewController.didSelectRelays = { [weak self] locations in

            guard let self else { return }

            var relayConstraints = tunnelManager.settings.relayConstraints
            relayConstraints.locations = .only(locations)

            tunnelManager.updateSettings([.relayConstraints(relayConstraints)]) {
                self.tunnelManager.startTunnel()
            }

            didFinish?(self)
        }

        locationViewController.navigateToFilter = { [weak self] in
            guard let self else { return }

            let coordinator = makeRelayFilterCoordinator(forModalPresentation: true)
            coordinator.start()

            presentChild(coordinator, animated: true)
        }

        locationViewController.didUpdateFilter = { [weak self] filter in
            guard let self else { return }

            var relayConstraints = tunnelManager.settings.relayConstraints
            relayConstraints.filter = .only(filter)

            tunnelManager.updateSettings([.relayConstraints(relayConstraints)])
        }

        locationViewController.didFinish = { [weak self] in
            guard let self else { return }

            didFinish?(self)
        }

        relayCacheTracker.addObserver(self)

        if let cachedRelays = try? relayCacheTracker.getCachedRelays() {
            self.cachedRelays = cachedRelays
            locationViewController.setCachedRelays(cachedRelays, filter: relayFilter)
        }

        locationViewController.relayLocations = tunnelManager.settings.relayConstraints.locations.value

        navigationController.pushViewController(locationViewController, animated: false)
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

extension LocationCoordinator: LocationViewControllerDelegate {
    func didRequestRouteToCustomLists(_ controller: LocationViewController) {
        let coordinator = AddCustomListCoordinator(
            navigationController: CustomNavigationController(),
            customListInteractor: CustomListInteractor(
                repository: customListRepository
            )
        )
        coordinator.start()
        presentChild(coordinator, animated: true)
    }
}
