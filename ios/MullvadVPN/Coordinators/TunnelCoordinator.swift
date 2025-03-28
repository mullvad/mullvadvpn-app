//
//  TunnelCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 01/02/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadREST
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
        outgoingConnectionService: OutgoingConnectionServiceHandling,
        ipOverrideRepository: IPOverrideRepositoryProtocol,
        relaySelectorWrapper: RelaySelectorWrapper
    ) {
        self.tunnelManager = tunnelManager

        let interactor = TunnelViewControllerInteractor(
            tunnelManager: tunnelManager,
            outgoingConnectionService: outgoingConnectionService,
            ipOverrideRepository: ipOverrideRepository
        )

        let relays = try! relaySelectorWrapper.relayCache.read()
        let relayLocations = RelayWithLocation.locateRelays(relays: relays.relays.wireguard.relays, locations: relays.relays.locations)

        controller = TunnelViewController(interactor: interactor, relays: relayLocations)

        super.init()

        controller.shouldShowSelectLocationPicker = { [weak self] in
            self?.showSelectLocationPicker?()
        }

        controller.shouldShowCancelTunnelAlert = { [weak self] in
            self?.showCancelTunnelAlert()
        }

        controller.didSelect = { [weak self] item in
            switch item.cell {
            case .location:
                var relayConstraints = tunnelManager.settings.relayConstraints
                relayConstraints.exitLocations = .only(.init(locations: [item.location!]))

                tunnelManager.updateSettings([.relayConstraints(relayConstraints)]) { [weak self] in
                    self?.tunnelManager.startTunnel()
                }
            case .setting:
                switch item.destination {
                case .daita:
                    self?.applicationRouter?.present(.daita)
                case .account:
                    self?.applicationRouter?.present(.account)
                case .selectLocation:
                    self?.applicationRouter?.present(.selectLocation)
                case .changelog:
                    self?.applicationRouter?.present(.changelog)
                case .multihop:
                    self?.applicationRouter?.present(.settings(.multihop))
                case .settings:
                    self?.applicationRouter?.present(.settings(nil))
                case .vpnSettings:
                    self?.applicationRouter?.present(.settings(.vpnSettings))
                case .problemReport:
                    self?.applicationRouter?.present(.settings(.problemReport))
                case .faq:
                    self?.applicationRouter?.present(.settings(.problemReport))
                case .apiAccess:
                    self?.applicationRouter?.present(.settings(.apiAccess))
                case .copyAccountNumber:
                    guard let accountData = tunnelManager.deviceState.accountData else { return }
                    UIPasteboard.general.string = accountData.number
                case .none:
                    break
                }
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
