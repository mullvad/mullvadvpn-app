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
    private var cachedRelays: LocationRelays?

    let navigationController: UINavigationController

    var presentedViewController: UIViewController {
        navigationController
    }

    var locationViewControllerWrapper: LocationViewControllerWrapper? {
        return navigationController.viewControllers.first {
            $0 is LocationViewControllerWrapper
        } as? LocationViewControllerWrapper
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
        let startContext: LocationViewControllerWrapper.MultihopContext =
            if case .noRelaysSatisfyingDaitaConstraints = tunnelManager.tunnelStatus.observedState
                .blockedState?.reason { .entry } else { .exit }

        let locationViewControllerWrapper = LocationViewControllerWrapper(
            customListRepository: customListRepository,
            constraints: tunnelManager.settings.relayConstraints,
            multihopEnabled: tunnelManager.settings.tunnelMultihopState.isEnabled,
            daitaEnabled: tunnelManager.settings.daita.daitaState.isEnabled,
            startContext: startContext
        )
        locationViewControllerWrapper.delegate = self

        locationViewControllerWrapper.didFinish = { [weak self] in
            guard let self else { return }
            didFinish?(self)
        }

        relayCacheTracker.addObserver(self)

        if let cachedRelays = try? relayCacheTracker.getCachedRelays() {
            let locationRelays = LocationRelays(
                relays: cachedRelays.relays.wireguard.relays,
                locations: cachedRelays.relays.locations
            )
            self.cachedRelays = locationRelays

            locationViewControllerWrapper.setCachedRelays(locationRelays, filter: relayFilter)
        }

        navigationController.pushViewController(locationViewControllerWrapper, animated: false)
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
            guard let self else { return }

            if var cachedRelays, let filter {
                cachedRelays.relays = cachedRelays.relays.filter { relay in
                    RelaySelector.relayMatchesFilter(relay, filter: filter)
                }

                locationViewControllerWrapper?.setCachedRelays(cachedRelays, filter: filter)
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
            self?.locationViewControllerWrapper?.refreshCustomLists()
        }

        coordinator.start()
        presentChild(coordinator, animated: true)
    }

    private func showEditCustomLists(nodes: [LocationNode]) {
        let coordinator = ListCustomListCoordinator(
            navigationController: InterceptibleNavigationController(),
            interactor: CustomListInteractor(repository: customListRepository),
            tunnelManager: tunnelManager,
            nodes: nodes
        )

        coordinator.didFinish = { [weak self] listCustomListCoordinator in
            listCustomListCoordinator.dismiss(animated: true)
            self?.locationViewControllerWrapper?.refreshCustomLists()
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
        locationViewControllerWrapper?.refreshCustomLists()
    }
}

extension LocationCoordinator: RelayCacheTrackerObserver {
    func relayCacheTracker(
        _ tracker: RelayCacheTracker,
        didUpdateCachedRelays cachedRelays: CachedRelays
    ) {
        let locationRelays = LocationRelays(
            relays: cachedRelays.relays.wireguard.relays,
            locations: cachedRelays.relays.locations
        )
        self.cachedRelays = locationRelays

        locationViewControllerWrapper?.setCachedRelays(locationRelays, filter: relayFilter)
    }
}

extension LocationCoordinator: LocationViewControllerWrapperDelegate {
    func didSelectEntryRelays(_ relays: UserSelectedRelays) {
        var relayConstraints = tunnelManager.settings.relayConstraints
        relayConstraints.entryLocations = .only(relays)

        tunnelManager.updateSettings([.relayConstraints(relayConstraints)]) {
            self.tunnelManager.startTunnel()
        }
    }

    func didSelectExitRelays(_ relays: UserSelectedRelays) {
        var relayConstraints = tunnelManager.settings.relayConstraints
        relayConstraints.exitLocations = .only(relays)

        tunnelManager.updateSettings([.relayConstraints(relayConstraints)]) {
            self.tunnelManager.startTunnel()
        }
    }

    func didUpdateFilter(_ filter: RelayFilter) {
        var relayConstraints = tunnelManager.settings.relayConstraints
        relayConstraints.filter = .only(filter)

        tunnelManager.updateSettings([.relayConstraints(relayConstraints)])

        if let cachedRelays {
            locationViewControllerWrapper?.setCachedRelays(cachedRelays, filter: filter)
        }
    }

    func navigateToFilter() {
        let coordinator = makeRelayFilterCoordinator(forModalPresentation: true)
        coordinator.start()

        presentChild(coordinator, animated: true)
    }

    func navigateToCustomLists(nodes: [LocationNode]) {
        let actionSheet = UIAlertController(
            title: NSLocalizedString(
                "ACTION_SHEET_TITLE", tableName: "CustomLists", value: "Custom lists", comment: ""
            ),
            message: nil,
            preferredStyle: UIDevice.current.userInterfaceIdiom == .pad ? .alert : .actionSheet
        )
        actionSheet.overrideUserInterfaceStyle = .dark
        actionSheet.view.tintColor = UIColor(red: 0.0, green: 0.59, blue: 1.0, alpha: 1)

        let addCustomListAction = UIAlertAction(
            title: NSLocalizedString(
                "ACTION_SHEET_ADD_LIST_BUTTON", tableName: "CustomLists", value: "Add new list", comment: ""
            ),
            style: .default,
            handler: { [weak self] _ in
                self?.showAddCustomList(nodes: nodes)
            }
        )
        addCustomListAction.accessibilityIdentifier = AccessibilityIdentifier.addNewCustomListButton
        actionSheet.addAction(addCustomListAction)

        let editAction = UIAlertAction(
            title: NSLocalizedString(
                "ACTION_SHEET_EDIT_LISTS_BUTTON", tableName: "CustomLists", value: "Edit lists", comment: ""
            ),
            style: .default,
            handler: { [weak self] _ in
                self?.showEditCustomLists(nodes: nodes)
            }
        )
        editAction.isEnabled = !customListRepository.fetchAll().isEmpty
        editAction.accessibilityIdentifier = AccessibilityIdentifier.editCustomListButton
        actionSheet.addAction(editAction)

        actionSheet.addAction(UIAlertAction(
            title: NSLocalizedString(
                "CUSTOM_LIST_ACTION_SHEET_CANCEL_BUTTON",
                tableName: "CustomLists",
                value: "Cancel",
                comment: ""
            ),
            style: .cancel
        ))

        presentationContext.present(actionSheet, animated: true)
    }
}
