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

class LocationCoordinator: Coordinator, Presentable, Presenting {
    private let tunnelManager: TunnelManager
    private let relayCacheTracker: RelayCacheTracker
    private let customListRepository: CustomListRepositoryProtocol
    private var cachedRelays: CachedRelays?

    let navigationController: UINavigationController

    var presentedViewController: UIViewController {
        navigationController
    }

    var locationViewController: LocationViewController? {
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
                self?.locationViewController?.setCachedRelays(cachedRelays, filter: filter)
            }

            coordinator.dismiss(animated: true)
        }

        return relayFilterCoordinator
    }

    private func showAddCustomList(nodes: [LocationNode]) {
        let coordinator = AddCustomListCoordinator(
            navigationController: CustomNavigationController(),
            interactor: CustomListInteractor(
                repository: customListRepository
            ),
            nodes: nodes
        )

        coordinator.didFinish = { [weak self] addCustomListCoordinator in
            addCustomListCoordinator.dismiss(animated: true)
            self?.locationViewController?.refreshCustomLists()
        }

        coordinator.start()
        presentChild(coordinator, animated: true)
    }

    private func showEditCustomLists(nodes: [LocationNode]) {
        let coordinator = ListCustomListCoordinator(
            navigationController: CustomNavigationController(),
            interactor: CustomListInteractor(repository: customListRepository),
            tunnelManager: tunnelManager,
            nodes: nodes
        )

        coordinator.didFinish = { [weak self] listCustomListCoordinator in
            listCustomListCoordinator.dismiss(animated: true)
            self?.locationViewController?.refreshCustomLists()
        }

        coordinator.start()
        presentChild(coordinator, animated: true)

        coordinator.presentedViewController.presentationController?.delegate = self
    }
}

// Intercept dismissal (by down swipe) of ListCustomListCoordinator and apply custom actions.
// See showEditCustomLists() above.
extension LocationCoordinator: UIAdaptivePresentationControllerDelegate {
    func presentationControllerDidDismiss(_ presentationController: UIPresentationController) {
        locationViewController?.refreshCustomLists()
    }
}

extension LocationCoordinator: RelayCacheTrackerObserver {
    func relayCacheTracker(
        _ tracker: RelayCacheTracker,
        didUpdateCachedRelays cachedRelays: CachedRelays
    ) {
        self.cachedRelays = cachedRelays

        locationViewController?.setCachedRelays(cachedRelays, filter: relayFilter)
    }
}

extension LocationCoordinator: LocationViewControllerDelegate {
    func didRequestRouteToCustomLists(_ controller: LocationViewController, nodes: [LocationNode]) {
        let actionSheet = UIAlertController(
            title: NSLocalizedString(
                "CUSTOM_LIST_ACTION_SHEET_TITLE",
                tableName: "CustomLists",
                value: "Custom lists",
                comment: ""
            ),
            message: nil,
            preferredStyle: UIDevice.current.userInterfaceIdiom == .pad ? .alert : .actionSheet
        )

        actionSheet.overrideUserInterfaceStyle = .dark
        actionSheet.view.tintColor = UIColor(red: 0.0, green: 0.59, blue: 1.0, alpha: 1)

        actionSheet.addAction(UIAlertAction(
            title: NSLocalizedString(
                "CUSTOM_LIST_ACTION_SHEET_ADD_LIST_BUTTON",
                tableName: "CustomLists",
                value: "Add new list",
                comment: ""
            ),
            style: .default,
            handler: { [weak self] _ in
                self?.showAddCustomList(nodes: nodes)
            }
        ))
        let editAction = UIAlertAction(
            title: NSLocalizedString(
                "CUSTOM_LIST_ACTION_SHEET_EDIT_LISTS_BUTTON",
                tableName: "CustomLists",
                value: "Edit lists",
                comment: ""
            ),
            style: .default,
            handler: { [weak self] _ in
                self?.showEditCustomLists(nodes: nodes)
            }
        )
        editAction.isEnabled = !customListRepository.fetchAll().isEmpty

        actionSheet.addAction(editAction)

        actionSheet.addAction(UIAlertAction(
            title: NSLocalizedString(
                "CUSTOM_LIST_ACTION_SHEET_CANCEL_BUTTON",
                tableName: "CustomLists",
                value: "Cancel",
                comment: ""
            ),
            style: .cancel,
            handler: nil
        ))

        presentationContext.present(actionSheet, animated: true)
    }
}
