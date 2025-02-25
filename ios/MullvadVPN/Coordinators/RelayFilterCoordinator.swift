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

class RelayFilterCoordinator: Coordinator, Presentable, @preconcurrency RelayCacheTrackerObserver {
    private let tunnelManager: TunnelManager
    private let relayCacheTracker: RelayCacheTracker
    private let relayFilterManager: RelayFilterable
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
        relayCacheTracker: RelayCacheTracker,
        relayFilterManager: RelayFilterable
    ) {
        self.navigationController = navigationController
        self.tunnelManager = tunnelManager
        self.relayCacheTracker = relayCacheTracker
        self.relayFilterManager = relayFilterManager
    }

    func start() {
        let locationRelays = if let cachedRelays = try? relayCacheTracker.getCachedRelays() {
            LocationRelays(
                relays: cachedRelays.relays.wireguard.relays,
                locations: cachedRelays.relays.locations
            )
        } else {
            LocationRelays(relays: [], locations: [:])
        }
        let relayFilterViewController = RelayFilterViewController(
            settings: tunnelManager.settings,
            relays: locationRelays,
            relayFilterManager: relayFilterManager
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
        addTunnelObserver()

        relayCacheTracker.addObserver(self)
        navigationController.pushViewController(relayFilterViewController, animated: false)
    }

    private func addTunnelObserver() {
        let tunnelObserver =
            TunnelBlockObserver(
                didUpdateTunnelSettings: { [weak self] _, settings in
                    guard let self else { return }
                    relayFilterViewController?.onNewSettings?(settings)
                }
            )

        tunnelManager.addObserver(tunnelObserver)
        self.tunnelObserver = tunnelObserver
    }

    func relayCacheTracker(
        _ tracker: RelayCacheTracker,
        didUpdateCachedRelays cachedRelays: CachedRelays
    ) {
        let locationRelays = if let cachedRelays = try? relayCacheTracker.getCachedRelays() {
            LocationRelays(
                relays: cachedRelays.relays.wireguard.relays,
                locations: cachedRelays.relays.locations
            )
        } else {
            LocationRelays(relays: [], locations: [:])
        }
        relayFilterViewController?.onNewRelays?(locationRelays)
    }
}
