//
//  TunnelCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 01/02/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
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
    var showFeatureSetting: ((AppRoute) -> Void)?

    init(
        tunnelManager: TunnelManager,
        outgoingConnectionService: OutgoingConnectionServiceHandling,
        ipOverrideRepository: IPOverrideRepositoryProtocol
    ) {
        self.tunnelManager = tunnelManager

        let interactor = TunnelViewControllerInteractor(
            tunnelManager: tunnelManager,
            outgoingConnectionService: outgoingConnectionService,
            ipOverrideRepository: ipOverrideRepository
        )

        controller = TunnelViewController(interactor: interactor)

        super.init()

        controller.shouldShowSelectLocationPicker = { [weak self] in
            self?.showSelectLocationPicker?()
        }

        controller.shouldShowCancelTunnelAlert = { [weak self] in
            self?.showCancelTunnelAlert()
        }

        controller.shouldShowSettingsForFeature = { [weak self] feature in
            switch feature {
            case .daita:
                self?.showFeatureSetting?(.daita)
            case .multihop:
                self?.showFeatureSetting?(.multihop)
            case .quantumResistance:
                self?.showFeatureSetting?(.vpnSettings(.quantumResistance))
            case .obfuscation:
                self?.showFeatureSetting?(.vpnSettings(.obfuscation))
            case .dns:
                self?.showFeatureSetting?(.dnsSettings)
            case .ipOverrides:
                self?.showFeatureSetting?(.ipOverrides)
            case .includeAllNetworks, .localNetworkSharing:
                self?.showFeatureSetting?(.includeAllNetworks)
            }
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
                "If you disconnect now, you won’t be able to secure your connection until the device is online.",
                comment: ""
            ),
            buttons: [
                AlertAction(
                    title: NSLocalizedString("Disconnect", comment: ""),
                    style: .destructive,
                    handler: { [weak self] in
                        self?.tunnelManager.stopTunnel()
                    }
                ),
                AlertAction(
                    title: NSLocalizedString("Cancel", comment: ""),
                    style: .default
                ),
            ]
        )

        let presenter = AlertPresenter(context: self)
        presenter.showAlert(presentation: presentation, animated: true)
    }
}
