//
//  TunnelCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 01/02/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Routing
import UIKit

class TunnelCoordinator: Coordinator {
    private let tunnelManager: TunnelManager
    private let controller: TunnelViewController
    private let alertPresenter = AlertPresenter()

    private var tunnelObserver: TunnelObserver?

    var rootViewController: UIViewController {
        controller
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

        controller.shouldShowCancelTunnelAlert = { [weak self] in
            self?.showCancelTunnelAlert()
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

    private func showCancelTunnelAlert() {
        let alertController = CustomAlertViewController(
            title: nil,
            message: NSLocalizedString(
                "CANCEL_TUNNEL_ALERT_MESSAGE",
                tableName: "Main",
                value: "If you disconnect now, you won’t be able to secure your connection until the device is online.",
                comment: ""
            ),
            icon: .alert
        )

        alertController.addAction(
            title: NSLocalizedString(
                "CANCEL_TUNNEL_ALERT_DISCONNECT_ACTION",
                tableName: "Main",
                value: "Disconnect",
                comment: ""
            ),
            style: .destructive,
            handler: { [weak self] in
                self?.tunnelManager.stopTunnel()
            }
        )

        alertController.addAction(
            title: NSLocalizedString(
                "CANCEL_TUNNEL_ALERT_CANCEL_ACTION",
                tableName: "Main",
                value: "Cancel",
                comment: ""
            ),
            style: .default
        )

        alertPresenter.enqueue(alertController, presentingController: rootViewController)
    }
}
