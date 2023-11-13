//
//  TunnelCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 01/02/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Routing
import UIKit

class TunnelCoordinator: Coordinator, Presenting {
    private let tunnelManager: TunnelManager
    private let controller: TunnelViewController

    private var tunnelObserver: TunnelObserver?

    var presentationContext: UIViewController {
        controller
    }

    var rootViewController: UIViewController {
        controller
    }

    var showSelectLocationPicker: (() -> Void)?

    init(
        tunnelManager: TunnelManager,
        outgoingConnectionService: OutgoingConnectionServiceHandling
    ) {
        self.tunnelManager = tunnelManager

        let interactor = TunnelViewControllerInteractor(
            tunnelManager: tunnelManager,
            outgoingConnectionService: outgoingConnectionService
        )
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
            TunnelBlockObserver(didUpdateDeviceState: { [weak self] _, _, _ in
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
        let presentation = AlertPresentation(
            id: "main-cancel-tunnel-alert",
            icon: .alert,
            message: NSLocalizedString(
                "CANCEL_TUNNEL_ALERT_MESSAGE",
                tableName: "Main",
                value: "If you disconnect now, you won’t be able to secure your connection until the device is online.",
                comment: ""
            ),
            buttons: [
                AlertAction(
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
                ),
                AlertAction(
                    title: NSLocalizedString(
                        "CANCEL_TUNNEL_ALERT_CANCEL_ACTION",
                        tableName: "Main",
                        value: "Cancel",
                        comment: ""
                    ),
                    style: .default
                ),
            ]
        )

        let presenter = AlertPresenter(context: self)
        presenter.showAlert(presentation: presentation, animated: true)
    }
}
