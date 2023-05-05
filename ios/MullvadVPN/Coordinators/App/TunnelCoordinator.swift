//
//  TunnelCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 01/02/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

class TunnelCoordinator: Coordinator {
    private let tunnelManager: TunnelManager
    private let controller: TunnelViewController

    private var tunnelObserver: TunnelObserver?

    var rootViewController: UIViewController {
        return controller
    }

    var showSelectLocationPicker: (() -> Void)?

    init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager

        let interactor = TunnelViewControllerInteractor(tunnelManager: tunnelManager)
        controller = TunnelViewController(interactor: interactor)

        super.init()

        controller.shouldShowSelectLocationPicker = { [weak self] in
            self?.showSelectLocationPicker?()
        }
    }

    func start() {
        let tunnelObserver =
            TunnelBlockObserver(didUpdateDeviceState: { [weak self] _, deviceState, previousDeviceState in
                self?.updateVisibility(animated: true)
            })

        self.tunnelObserver = tunnelObserver

        tunnelManager.addObserver(tunnelObserver)

        updateVisibility(animated: false)
    }

    private func updateVisibility(animated: Bool) {
        let deviceState = tunnelManager.deviceState

        controller.setMainContentHidden(!deviceState.isLoggedIn, animated: animated)
    }
}
