//
//  ListCustomListCoordinator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-03-06.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes
import Routing
import UIKit

class ListCustomListCoordinator: Coordinator, Presentable, Presenting {
    let navigationController: UINavigationController
    let interactor: CustomListInteractorProtocol
    let tunnelManager: TunnelManager
    let listViewController: ListCustomListViewController
    let nodes: [LocationNode]

    var presentedViewController: UIViewController {
        navigationController
    }

    var didFinish: ((ListCustomListCoordinator) -> Void)?

    init(
        navigationController: UINavigationController,
        interactor: CustomListInteractorProtocol,
        tunnelManager: TunnelManager,
        nodes: [LocationNode]
    ) {
        self.navigationController = navigationController
        self.interactor = interactor
        self.tunnelManager = tunnelManager
        self.nodes = nodes

        listViewController = ListCustomListViewController(interactor: interactor)
    }

    func start() {
        listViewController.didFinish = { [weak self] in
            guard let self else { return }
            didFinish?(self)
        }
        listViewController.didSelectItem = { [weak self] in
            self?.edit(list: $0)
        }

        navigationController.pushViewController(listViewController, animated: false)
    }

    private func edit(list: CustomList) {
        let coordinator = EditCustomListCoordinator(
            navigationController: navigationController,
            customListInteractor: interactor,
            customList: list,
            nodes: nodes
        )

        coordinator.didFinish = { [weak self] editCustomListCoordinator, action, list in
            guard let self else { return }

            popToList()
            editCustomListCoordinator.removeFromParent()

            var relayConstraints = tunnelManager.settings.relayConstraints
            relayConstraints.entryLocations = self.updateRelayConstraint(
                relayConstraints.entryLocations,
                for: action,
                in: list
            )
            relayConstraints.exitLocations = self.updateRelayConstraint(
                relayConstraints.exitLocations,
                for: action,
                in: list
            )

            tunnelManager.updateSettings([.relayConstraints(relayConstraints)]) { [weak self] in
                self?.tunnelManager.reconnectTunnel(selectNewRelay: true)
            }
        }

        coordinator.didCancel = { [weak self] editCustomListCoordinator in
            guard let self else { return }
            popToList()
            editCustomListCoordinator.removeFromParent()
        }

        coordinator.start()
        addChild(coordinator)
    }

    private func updateRelayConstraint(
        _ relayConstraint: RelayConstraint<UserSelectedRelays>,
        for action: EditCustomListCoordinator.FinishAction,
        in list: CustomList
    ) -> RelayConstraint<UserSelectedRelays> {
        var relayConstraint = relayConstraint

        guard let customListSelection = relayConstraint.value?.customListSelection,
              customListSelection.listId == list.id
        else { return relayConstraint }

        switch action {
        case .save:
            if customListSelection.isList {
                let selectedRelays = UserSelectedRelays(
                    locations: list.locations,
                    customListSelection: UserSelectedRelays.CustomListSelection(listId: list.id, isList: true)
                )
                relayConstraint = .only(selectedRelays)
            } else {
                let selectedConstraintIsRemovedFromList = list.locations.filter {
                    relayConstraint.value?.locations.contains($0) ?? false
                }.isEmpty

                if selectedConstraintIsRemovedFromList {
                    relayConstraint = .only(UserSelectedRelays(locations: []))
                }
            }
        case .delete:
            relayConstraint = .only(UserSelectedRelays(locations: []))
        }

        return relayConstraint
    }

    private func popToList() {
        if interactor.fetchAll().isEmpty {
            navigationController.dismiss(animated: true)
            didFinish?(self)
        } else if let listController = navigationController.viewControllers
            .first(where: { $0 is ListCustomListViewController }) {
            navigationController.popToViewController(listController, animated: true)
            listViewController.updateDataSource(reloadExisting: true, animated: false)
        }
    }
}
