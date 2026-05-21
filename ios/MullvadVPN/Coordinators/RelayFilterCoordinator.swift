//
//  RelayFilterCoordinator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-06-14.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadTypes
import Routing
import UIKit

class RelayFilterCoordinator: Coordinator, Presentable {
    private let tunnelManager: TunnelManager
    private let relaySelectorWrapper: RelaySelectorWrapper
    private let multihopContext: MultihopContext

    let navigationController: UINavigationController

    var presentedViewController: UIViewController {
        return navigationController
    }

    var relayFilterViewController: RelayFilterSelection.ViewController? {
        return navigationController.viewControllers.first {
            $0 is RelayFilterSelection.ViewController
        } as? RelayFilterSelection.ViewController
    }

    var didFinish: ((RelayFilterCoordinator, RelayFilter?) -> Void)?
    var onFeatureChipTapped: ((FeatureType) -> Void)?

    init(
        navigationController: UINavigationController,
        tunnelManager: TunnelManager,
        multihopContext: MultihopContext,
        relaySelectorWrapper: RelaySelectorWrapper
    ) {
        self.navigationController = navigationController
        self.tunnelManager = tunnelManager
        self.multihopContext = multihopContext
        self.relaySelectorWrapper = relaySelectorWrapper
    }

    func start() {

        let relayFilterViewModel = RelayFilterSelection.ViewModel(
            tunnelManager: tunnelManager,
            relaySelectorWrapper: relaySelectorWrapper,
            multihopContext: multihopContext
        )
        relayFilterViewModel.onFeatureChipTapped = { [weak self] feature in
            self?.onFeatureChipTapped?(feature)
        }
        let relayFilterViewController = RelayFilterSelection.ViewController(viewModel: relayFilterViewModel)

        relayFilterViewController.onApplyFilter = { [weak self] filter, multihopContext in
            guard let self else { return }

            var relayConstraints = tunnelManager.settings.relayConstraints
            relayConstraints.setFilterConstraint(.only(filter), for: multihopContext)
            tunnelManager.updateSettings([.relayConstraints(relayConstraints)])

            didFinish?(self, filter)
        }

        relayFilterViewController.didFinish = { [weak self] in
            guard let self else { return }
            didFinish?(self, nil)
        }
        navigationController.pushViewController(relayFilterViewController, animated: false)
    }
}
