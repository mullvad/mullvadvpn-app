//
//  LaunchViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 18/11/2021.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import UIKit

class LaunchViewController: UIViewController {
    private let tunnelManager: TunnelManager
    private var isFirstLaunch: Bool = true
    private var tunnelObserver: TunnelObserver!
    private let launchArguments: LaunchArguments

    var onAppReady: (() -> Void)?

    init(launchArguments: LaunchArguments, tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager
        self.launchArguments = launchArguments
        super.init(nibName: nil, bundle: nil)
        setupLaunchScreen()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override var preferredStatusBarStyle: UIStatusBarStyle {
        .lightContent
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        addTunnelObserver()
    }

    private func addTunnelObserver() {
        guard launchArguments.isResetAppAllowed else { return }
        let tunnelObserver =
            TunnelBlockObserver(
                didLoadConfiguration: { [weak self] tunnelManager in
                    guard let self else { return }
                    Task {
                        await reset()
                    }
                },
                didUpdateTunnelStatus: { [weak self] _, tunnelStatus in
                    guard let self else { return }
                    if case .connected = tunnelStatus.observedState {
                        tunnelManager.stopTunnel()
                    } else if case .disconnected = tunnelStatus.observedState {
                        Task {
                            await reset()
                        }
                    }
                })

        tunnelManager.addObserver(tunnelObserver)
        self.tunnelObserver = tunnelObserver
    }

    private func reset() async {
        defer {
            tunnelManager.removeObserver(self.tunnelObserver)
            onAppReady?()
        }
        guard launchArguments.isResetAppAllowed, isFirstLaunch else {
            return
        }
        isFirstLaunch = false
        await tunnelManager.unsetAccount()
        let settings = LatestTunnelSettings()
        tunnelManager.updateSettings([
            .relayConstraints(settings.relayConstraints), .dnsSettings(settings.dnsSettings),
            .daita(settings.daita), .includeAllNetworks(settings.includeAllNetworks),
            .multihop(settings.tunnelMultihopState), .quantumResistance(settings.tunnelQuantumResistance),.obfuscation(settings.wireGuardObfuscation)
        ])
       
    }

    private func setupLaunchScreen() {
        let storyboard = UIStoryboard(name: "LaunchScreen", bundle: nil)

        guard let initialController = storyboard.instantiateInitialViewController() else {
            assertionFailure("LaunchScreen storyboard misconfigured")
            return
        }

        initialController.view.translatesAutoresizingMaskIntoConstraints = false

        addChild(initialController)
        view.addSubview(initialController.view)
        initialController.didMove(toParent: self)

        NSLayoutConstraint.activate([
            initialController.view.topAnchor.constraint(equalTo: view.topAnchor),
            initialController.view.bottomAnchor.constraint(equalTo: view.bottomAnchor),
            initialController.view.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            initialController.view.trailingAnchor.constraint(equalTo: view.trailingAnchor),
        ])
    }

    private func prepareApp(launchArguments: LaunchArguments) async {
        if launchArguments.isResetAppAllowed {
            await tunnelManager.unsetAccount()
        }

        isFirstLaunch = false
    }

}
