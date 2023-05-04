//
//  PreferencesInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 31/10/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import RelayCache

final class PreferencesInteractor {
    private let tunnelManager: TunnelManager
    private var tunnelObserver: TunnelObserver?
    private let relayCacheTracker: RelayCacheTracker

    var tunnelSettingsDidChange: ((TunnelSettingsV2) -> Void)?
    var cachedRelaysDidChange: ((CachedRelays) -> Void)?

    var tunnelSettings: TunnelSettingsV2 {
        return tunnelManager.settings
    }

    var cachedRelays: CachedRelays? {
        return try? relayCacheTracker.getCachedRelays()
    }

    init(tunnelManager: TunnelManager, relayCacheTracker: RelayCacheTracker) {
        self.tunnelManager = tunnelManager
        self.relayCacheTracker = relayCacheTracker

        tunnelObserver =
            TunnelBlockObserver(didUpdateTunnelSettings: { [weak self] manager, newSettings in
                self?.tunnelSettingsDidChange?(newSettings)
            })
    }

    func setDNSSettings(_ newDNSSettings: DNSSettings, completion: (() -> Void)? = nil) {
        tunnelManager.setDNSSettings(newDNSSettings, completionHandler: completion)
    }

    func setPort(_ port: UInt16?, completion: (() -> Void)? = nil) {
        var relayConstraints = tunnelManager.settings.relayConstraints

        if let port {
            relayConstraints.port = .only(port)
        } else {
            relayConstraints.port = .any
        }

        tunnelManager.setRelayConstraints(relayConstraints, completionHandler: completion)
    }
}

extension PreferencesInteractor: RelayCacheTrackerObserver {
    func relayCacheTracker(_ tracker: RelayCacheTracker, didUpdateCachedRelays cachedRelays: CachedRelays) {
        cachedRelaysDidChange?(cachedRelays)
    }
}
