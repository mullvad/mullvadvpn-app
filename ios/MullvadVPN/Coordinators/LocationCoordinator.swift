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
import SwiftUI

class LocationCoordinator: Coordinator, Presentable, Presenting {
    private let tunnelManager: TunnelManager
    private var tunnelObserver: TunnelObserver?
    private let relaySelectorWrapper: RelaySelectorWrapper
    private let customListRepository: CustomListRepositoryProtocol

    let navigationController: UINavigationController

    var presentedViewController: UIViewController {
        navigationController
    }

//    var locationViewControllerWrapper: LocationViewControllerWrapper? {
//        return navigationController.viewControllers.first {
//            $0 is LocationViewControllerWrapper
//        } as? LocationViewControllerWrapper
//    }

    var selectLocationViewModel: (any SelectLocationViewModel)? {
        (navigationController.viewControllers.first {
            $0 is UIHostingController<SelectLocationView<SelectLocationViewModelImpl>>
        } as? UIHostingController<SelectLocationView<SelectLocationViewModelImpl>>)?.rootView.viewModel
    }

    var didFinish: ((LocationCoordinator) -> Void)?

    init(
        navigationController: UINavigationController,
        tunnelManager: TunnelManager,
        relaySelectorWrapper: RelaySelectorWrapper,
        customListRepository: CustomListRepositoryProtocol
    ) {
        self.navigationController = navigationController
        self.tunnelManager = tunnelManager
        self.relaySelectorWrapper = relaySelectorWrapper
        self.customListRepository = customListRepository
    }

    func start() {
        let hostingController = UIHostingController(
            rootView: SelectLocationView(
                viewModel: SelectLocationViewModelImpl(
                    tunnelManager: tunnelManager,
                    relaySelectorWrapper: relaySelectorWrapper,
                    customListRepository: customListRepository,
                    didSelectExitRelayLocations: { [weak self] relays in
                        guard let self else { return }
                        self.didSelectExitRelays(relays)
                        self.didFinish?(self)
                    },
                    didSelectEntryRelayLocations: { [weak self] relays in
                        guard let self else { return }
                        self.didSelectEntryRelays(relays)
                        self.didFinish?(self)
                    },
                    showFilterView: { [weak self] in
                        guard let self else { return }
                        self.navigateToFilter()
                    },
                    showEditCustomListView: { [weak self] locations in
                        guard let self else { return }
                        self.showEditCustomLists(nodes: locations)
                    },
                    showAddCustomListView: { [weak self] locations in
                        guard let self else { return }
                        self.showAddCustomList(nodes: locations)
                    },
                    didFinish: { [weak self] in
                        guard let self else { return }
                        self.didFinish?(self)
                    }
                )
            )
        )
        navigationController.pushViewController(hostingController, animated: false)
    }

//    private func addTunnelObserver() {
//        let tunnelObserver =
//            TunnelBlockObserver(
//                didUpdateTunnelSettings: { [weak self] _, settings in
//                    guard let self else { return }
//                    locationViewControllerWrapper?.onNewSettings?(settings)
//                }
//            )
//
//        tunnelManager.addObserver(tunnelObserver)
//        self.tunnelObserver = tunnelObserver
//    }

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
//            self?.locationViewControllerWrapper?.refreshCustomLists()
            self?.selectLocationViewModel?.refreshCustomLists()
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
//            self?.locationViewControllerWrapper?.refreshCustomLists()
            self?.selectLocationViewModel?.refreshCustomLists()
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
//        locationViewControllerWrapper?.refreshCustomLists()
        selectLocationViewModel?.refreshCustomLists()
    }
}

extension LocationCoordinator {
    func navigateToFilter() {
        let relayFilterCoordinator = RelayFilterCoordinator(
            navigationController: CustomNavigationController(),
            tunnelManager: tunnelManager,
            relaySelectorWrapper: relaySelectorWrapper
        )

        relayFilterCoordinator.didFinish = { coordinator, _ in
            coordinator.dismiss(animated: true)
        }
        relayFilterCoordinator.start()

        presentChild(relayFilterCoordinator, animated: true)
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
    }

    func navigateToCustomLists(nodes: [LocationNode]) {
        let actionSheet = UIAlertController(
            title: NSLocalizedString("Custom lists", comment: ""),
            message: nil,
            preferredStyle: UIDevice.current.userInterfaceIdiom == .pad ? .alert : .actionSheet
        )
        actionSheet.overrideUserInterfaceStyle = .dark
        actionSheet.view.tintColor = .AlertController.tintColor

        let addCustomListAction = UIAlertAction(
            title: NSLocalizedString("Add new list", comment: ""),
            style: .default,
            handler: { [weak self] _ in
                self?.showAddCustomList(nodes: nodes)
            }
        )
        addCustomListAction.setAccessibilityIdentifier(.addNewCustomListButton)
        actionSheet.addAction(addCustomListAction)

        let editAction = UIAlertAction(
            title: NSLocalizedString("Edit lists", comment: ""),
            style: .default,
            handler: { [weak self] _ in
                self?.showEditCustomLists(nodes: nodes)
            }
        )
        editAction.isEnabled = !customListRepository.fetchAll().isEmpty
        editAction.setAccessibilityIdentifier(.editCustomListButton)
        actionSheet.addAction(editAction)

        actionSheet.addAction(UIAlertAction(
            title: NSLocalizedString("Cancel", comment: ""),
            style: .cancel
        ))

        presentationContext.present(actionSheet, animated: true)
    }
}
