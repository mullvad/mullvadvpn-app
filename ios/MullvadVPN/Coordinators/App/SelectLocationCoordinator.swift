//
//  SelectLocationCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 29/01/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
import RelayCache
import UIKit

class SelectLocationCoordinator: Coordinator, Presentable, RelayCacheTrackerObserver {
    let navigationController: UINavigationController

    var presentedViewController: UIViewController {
        return navigationController
    }

    private let tunnelManager: TunnelManager
    private let relayCacheTracker: RelayCacheTracker

    var didFinish: ((SelectLocationCoordinator, RelayLocation?) -> Void)?

    init(
        navigationController: UINavigationController,
        tunnelManager: TunnelManager,
        relayCacheTracker: RelayCacheTracker
    ) {
        self.navigationController = navigationController
        self.tunnelManager = tunnelManager
        self.relayCacheTracker = relayCacheTracker
    }

    func start() {
        let controller = SelectLocationViewController()

        controller.didSelectRelay = { [weak self] _, relay in
            guard let self = self else { return }

            let newConstraints = RelayConstraints(location: .only(relay))

            self.tunnelManager.updateSettings(request: .init(relayConstraints: newConstraints)) {
                self.tunnelManager.startTunnel()
            }

            self.didFinish?(self, relay)
        }

        controller.didFinish = { [weak self] _ in
            guard let self = self else { return }

            self.didFinish?(self, nil)
        }

        relayCacheTracker.addObserver(self)

        if let cachedRelays = try? relayCacheTracker.getCachedRelays() {
            controller.setCachedRelays(cachedRelays)
        }

        let relayConstraints = tunnelManager.settings.relayConstraints

        controller.setSelectedRelayLocation(
            relayConstraints.location.value,
            animated: false,
            scrollPosition: .middle
        )

        navigationController.pushViewController(controller, animated: false)
    }

    func relayCacheTracker(
        _ tracker: RelayCacheTracker,
        didUpdateCachedRelays cachedRelays: CachedRelays
    ) {
        guard let controller = navigationController.viewControllers
            .first as? SelectLocationViewController else { return }

        controller.setCachedRelays(cachedRelays)
    }
}
