//
//  RelayFilterCoordinator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-06-14.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadTypes
import Routing
import UIKit

class RelayFilterCoordinator: Coordinator, Presentable {
    private let tunnelManager: TunnelManager
    private let relaySelectorWrapper: RelaySelectorWrapper
    private var tunnelObserver: TunnelObserver?

    let navigationController: UINavigationController

    var presentedViewController: UIViewController {
        return navigationController
    }

    var relayFilterViewController: RelayFilterViewController? {
        return navigationController.viewControllers.first {
            $0 is RelayFilterViewController
        } as? RelayFilterViewController
    }

    var didFinish: ((RelayFilterCoordinator, RelayFilter?) -> Void)?

    init(
        navigationController: UINavigationController,
        tunnelManager: TunnelManager,
        relaySelectorWrapper: RelaySelectorWrapper
    ) {
        self.navigationController = navigationController
        self.tunnelManager = tunnelManager
        self.relaySelectorWrapper = relaySelectorWrapper
    }

    func start() {
        let relayFilterViewController = RelayFilterViewController(
            settings: tunnelManager.settings,
            relaySelectorWrapper: relaySelectorWrapper
        )

        relayFilterViewController.onApplyFilter = { [weak self] filter in
            guard let self else { return }

            var relayConstraints = tunnelManager.settings.relayConstraints
            relayConstraints.filter = .only(filter)

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
