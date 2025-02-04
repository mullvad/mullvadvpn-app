//
//  VPNSettingsInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 31/10/2022.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadSettings

final class VPNSettingsInteractor {
    let tunnelManager: TunnelManager
    private var tunnelObserver: TunnelObserver?
    private let relayCacheTracker: RelayCacheTracker

    var tunnelSettingsDidChange: ((LatestTunnelSettings) -> Void)?
    var cachedRelaysDidChange: ((CachedRelays) -> Void)?

    var tunnelSettings: LatestTunnelSettings {
        tunnelManager.settings
    }

    var cachedRelays: CachedRelays? {
        try? relayCacheTracker.getCachedRelays()
    }

    init(tunnelManager: TunnelManager, relayCacheTracker: RelayCacheTracker) {
        self.tunnelManager = tunnelManager
        self.relayCacheTracker = relayCacheTracker

        let tunnelObserver =
            TunnelBlockObserver(didUpdateTunnelSettings: { [weak self] _, newSettings in
                self?.tunnelSettingsDidChange?(newSettings)
            })
        self.tunnelObserver = tunnelObserver
        tunnelManager.addObserver(tunnelObserver)
    }

    func updateSettings(_ changes: [TunnelSettingsUpdate], completion: (@Sendable () -> Void)? = nil) {
        tunnelManager.updateSettings(changes, completionHandler: completion)
    }

    func setPort(_ port: UInt16?, completion: (@Sendable () -> Void)? = nil) {
        var relayConstraints = tunnelManager.settings.relayConstraints

        if let port {
            relayConstraints.port = .only(port)
        } else {
            relayConstraints.port = .any
        }

        tunnelManager.updateSettings([.relayConstraints(relayConstraints)], completionHandler: completion)
    }
}

extension VPNSettingsInteractor: RelayCacheTrackerObserver {
    func relayCacheTracker(_ tracker: RelayCacheTracker, didUpdateCachedRelays cachedRelays: CachedRelays) {
        cachedRelaysDidChange?(cachedRelays)
    }
}
