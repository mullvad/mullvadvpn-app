//
//  LocationCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 29/01/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadSettings
import MullvadTypes
import Routing
import UIKit

class LocationCoordinator: Coordinator, Presentable, Presenting {
    private let tunnelManager: TunnelManager
    private var tunnelObserver: TunnelObserver?
    private let relayCacheTracker: RelayCacheTracker
    private let customListRepository: CustomListRepositoryProtocol

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
        // If multihop is enabled, we should check if there's a DAITA related error when opening the location
        // view. If there is, help the user by showing the entry instead of the exit view.
//        var startContext: LocationViewControllerWrapper.MultihopContext = .exit
//        let relays =
//        if tunnelManager.settings.tunnelMultihopState.isEnabled {
//            startContext = if case .noRelaysSatisfyingDaitaConstraints = tunnelManager.tunnelStatus.observedState
//                .blockedState?.reason { .entry } else { .exit }
//        }

        let locationRelays = if let cachedRelays = try? relayCacheTracker.getCachedRelays() {
            LocationRelays(
                relays: cachedRelays.relays.wireguard.relays,
                locations: cachedRelays.relays.locations
            )
        } else {
            LocationRelays(relays: [], locations: [:])
        }

        let locationViewControllerWrapper = LocationViewControllerWrapper(
            settings: tunnelManager.settings,
            relays: locationRelays,
            customListRepository: customListRepository
        )

//        let locationViewControllerWrapper = LocationViewControllerWrapper(
//            customListRepository: customListRepository,
//            constraints: tunnelManager.settings.relayConstraints,
//            multihopEnabled: tunnelManager.settings.tunnelMultihopState.isEnabled,
//            daitaSettings: tunnelManager.settings.daita,
//            startContext: startContext
//        )
        locationViewControllerWrapper.delegate = self

        locationViewControllerWrapper.didFinish = { [weak self] in
            guard let self else { return }

            if let tunnelObserver {
                tunnelManager.removeObserver(tunnelObserver)
            }
            didFinish?(self)
        }

        addTunnelObserver()
        relayCacheTracker.addObserver(self)

        navigationController.pushViewController(locationViewControllerWrapper, animated: false)
    }

    private func addTunnelObserver() {
        let tunnelObserver =
            TunnelBlockObserver(
                didUpdateTunnelSettings: { [weak self] _, settings in
                    guard let self else { return }
                    locationViewControllerWrapper?.onNewSettings?(settings)
                }
            )

        tunnelManager.addObserver(tunnelObserver)
        self.tunnelObserver = tunnelObserver
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

extension LocationCoordinator: @preconcurrency RelayCacheTrackerObserver {
    func relayCacheTracker(
        _ tracker: RelayCacheTracker,
        didUpdateCachedRelays cachedRelays: CachedRelays
    ) {
        locationViewControllerWrapper?.onNewRelays?(LocationRelays(
            relays: cachedRelays.relays.wireguard.relays,
            locations: cachedRelays.relays.locations
        ))
    }
}

extension LocationCoordinator: @preconcurrency LocationViewControllerWrapperDelegate {
    func navigateToFilter(_ relayFilterManager: RelayFilterManager) {
        <#code#>
    }
    
    func didSelectEntryRelays(_ relays: UserSelectedRelays) {
        var relayConstraints = tunnelManager.settings.relayConstraints
        relayConstraints.entryLocations = .only(relays)

        tunnelManager.updateSettings([.relayConstraints(relayConstraints)]) {
            self.tunnelManager.startTunnel()
        }
    }

    func navigateToDaitaSettings() {
        applicationRouter?.present(.daita)
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

//        if let cachedRelays = try? relayCacheTracker.getCachedRelays(), let locationViewControllerWrapper {
//            locationViewControllerWrapper.setRelaysWithLocation(LocationRelays(
//                relays: cachedRelays.relays.wireguard.relays,
//                locations: cachedRelays.relays.locations
//            ), filter: filter)
//        }
    }

    func navigateToFilter() {
        let navigationController = CustomNavigationController()

        let relayFilterCoordinator = RelayFilterCoordinator(
            navigationController: navigationController,
            tunnelManager: tunnelManager,
            relayCacheTracker: relayCacheTracker
        )

        relayFilterCoordinator.didFinish = { [weak self] coordinator, filter in
            guard let self else { return }

//            if let cachedRelays = try? relayCacheTracker.getCachedRelays(), let locationViewControllerWrapper,
//               let filter {
//                locationViewControllerWrapper.setRelaysWithLocation(LocationRelays(
//                    relays: cachedRelays.relays.wireguard.relays,
//                    locations: cachedRelays.relays.locations
//                ), filter: filter)
//            }

            coordinator.dismiss(animated: true)
        }

        return relayFilterCoordinator
//        let coordinator = makeRelayFilterCoordinator(forModalPresentation: true)
//        coordinator.start()
//
//        presentChild(coordinator, animated: true)
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
        actionSheet.view.tintColor = .AlertController.tintColor

        let addCustomListAction = UIAlertAction(
            title: NSLocalizedString(
                "ACTION_SHEET_ADD_LIST_BUTTON", tableName: "CustomLists", value: "Add new list", comment: ""
            ),
            style: .default,
            handler: { [weak self] _ in
                self?.showAddCustomList(nodes: nodes)
            }
        )
        addCustomListAction.setAccessibilityIdentifier(.addNewCustomListButton)
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
        editAction.setAccessibilityIdentifier(.editCustomListButton)
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
