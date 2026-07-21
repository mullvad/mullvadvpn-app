//
//  LocationCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 29/01/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
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
    private let relayCacheTracker: RelayCacheTrackerProtocol
    private let customListRepository: CustomListRepositoryProtocol
    private let recentConnectionsRepository: RecentConnectionsRepositoryProtocol

    let navigationController: UINavigationController

    var presentedViewController: UIViewController {
        navigationController
    }

    var selectLocationViewModel: (any SelectLocationViewModel)!

    var didFinish: ((LocationCoordinator) -> Void)?

    init(
        navigationController: UINavigationController,
        tunnelManager: TunnelManager,
        relaySelectorWrapper: RelaySelectorWrapper,
        relayCacheTracker: RelayCacheTrackerProtocol,
        customListRepository: CustomListRepositoryProtocol,
        recentConnectionsRepository: RecentConnectionsRepositoryProtocol
    ) {
        self.navigationController = navigationController
        self.tunnelManager = tunnelManager
        self.relaySelectorWrapper = relaySelectorWrapper
        self.relayCacheTracker = relayCacheTracker
        self.customListRepository = customListRepository
        self.recentConnectionsRepository = recentConnectionsRepository
    }

    func start() {
        let selectLocationViewModelImpl = SelectLocationViewModelImpl(
            tunnelManager: tunnelManager,
            relaySelectorWrapper: relaySelectorWrapper,
            relayCacheTracker: relayCacheTracker,
            customListRepository: customListRepository,
            recentConnectionsRepository: recentConnectionsRepository,
            delegate: .init(
                showDaitaSettings: { [weak self] in
                    self?.navigateToDaitaSettings()
                },
                showObfuscationSettings: { [weak self] in
                    self?.navigateToObfuscationSettings()
                },
                showIpVersionSettings: { [weak self] in
                    self?.navigateToIpVersionSettings()
                },
                showFilterView: { [weak self] multihopContext in
                    self?.navigateToFilter(multihopContext: multihopContext)
                },
                showEditCustomListView: { [weak self] locations, customList in
                    if let customList {
                        self?.showEditCustomList(
                            list: customList,
                            nodes: locations
                        )
                    } else {
                        self?.showEditCustomLists(nodes: locations)
                    }
                },
                showAddCustomListView: { [weak self] locations in
                    self?.showAddCustomList(nodes: locations)
                },
                didSelectExitRelayLocations: { [weak self] constraint in
                    guard let self else { return }
                    self.didSelectExitRelays(constraint)
                    self.didFinish?(self)
                },
                didSelectEntryRelayLocations: { [weak self] constraint in
                    self?.didSelectEntryRelays(constraint)
                },
                didFinish: { [weak self] in
                    guard let self else { return }
                    self.didFinish?(self)
                }
            )
        )
        selectLocationViewModel = selectLocationViewModelImpl
        let hostingController = UIHostingController(
            rootView: SelectLocationView(
                viewModel: selectLocationViewModelImpl)
        )
        hostingController.view.setAccessibilityIdentifier(.selectLocationView)

        navigationController.pushViewController(hostingController, animated: false)
    }

    private func showAddCustomList(nodes: [LocationNode]) {
        let coordinator = AddCustomListCoordinator(
            navigationController: CustomNavigationController(),
            interactor: selectLocationViewModel,
            nodes: nodes
        )

        coordinator.didFinish = { [weak self] addCustomListCoordinator in
            addCustomListCoordinator.dismiss(animated: true)
            self?.selectLocationViewModel?.customListsChanged()
        }

        coordinator.start()
        presentChild(coordinator, animated: true)
    }

    private func showEditCustomLists(nodes: [LocationNode]) {
        let coordinator = ListCustomListCoordinator(
            navigationController: InterceptibleNavigationController(),
            interactor: selectLocationViewModel,
            tunnelManager: tunnelManager,
            nodes: nodes,
        )

        coordinator.didFinish = { [weak self] listCustomListCoordinator, action in
            guard let self else { return }
            listCustomListCoordinator.dismiss(animated: true)

            switch action {
            case .didDelete(let list):
                self.selectLocationViewModel.delete(customList: list)
            case .didSave(let list):
                try? self.selectLocationViewModel.save(list: list)
            case .noAction:
                self.selectLocationViewModel.customListsChanged()
            }
        }

        coordinator.start()
        presentChild(coordinator, animated: true)

        coordinator.presentedViewController.presentationController?.delegate = self
    }

    private func showEditCustomList(list: CustomList, nodes: [LocationNode]) {
        let coordinator = EditCustomListCoordinator(
            navigationController: InterceptibleNavigationController(),
            customListInteractor: selectLocationViewModel,
            customList: list,
            nodes: nodes
        )

        coordinator.didFinish = { [weak self] editCustomListCoordinator, _ in
            editCustomListCoordinator.dismiss(animated: true)
            self?.selectLocationViewModel?.customListsChanged()
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
        selectLocationViewModel?.customListsChanged()
    }
}

extension LocationCoordinator {
    func navigateToFilter(multihopContext: MultihopContext) {
        let relayFilterCoordinator = RelayFilterCoordinator(
            navigationController: CustomNavigationController(),
            tunnelManager: tunnelManager,
            multihopContext: multihopContext,
            relaySelectorWrapper: relaySelectorWrapper
        )

        relayFilterCoordinator.didFinish = {
            relayFilterCoordinator.dismiss(animated: true)
        }
        relayFilterCoordinator.onFeatureChipTapped = { [weak self] feature in
            switch feature {
            case .daita: self?.navigateToDaitaSettings()
            case .obfuscation: self?.navigateToObfuscationSettings()
            default: break
            }
        }
        relayFilterCoordinator.start()

        presentChild(relayFilterCoordinator, animated: true)
    }

    func didSelectEntryRelays(_ constraint: RelayConstraint<UserSelectedRelays>) {
        var relayConstraints = tunnelManager.settings.relayConstraints
        relayConstraints.entryLocations = constraint

        tunnelManager.updateSettings([.relayConstraints(relayConstraints)]) {
            self.tunnelManager.startTunnel()
        }
    }

    func navigateToDaitaSettings() {
        applicationRouter?.present(.daita)
    }

    func navigateToObfuscationSettings() {
        applicationRouter?.present(.vpnSettings(.obfuscation))
    }

    func navigateToIpVersionSettings() {
        applicationRouter?.present(.vpnSettings(.ipVersion))
    }

    func didSelectExitRelays(_ constraint: RelayConstraint<UserSelectedRelays>) {
        var relayConstraints = tunnelManager.settings.relayConstraints
        relayConstraints.exitLocations = constraint

        tunnelManager.updateSettings([.relayConstraints(relayConstraints)]) {
            self.tunnelManager.startTunnel()
        }
    }
}
