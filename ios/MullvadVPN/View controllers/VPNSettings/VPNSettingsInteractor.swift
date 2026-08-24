// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadREST
import MullvadSettings

final class VPNSettingsInteractor {
    let tunnelManager: TunnelManager
    private var tunnelObserver: TunnelObserver?
    private let relayCacheTracker: RelayCacheTracker

    var tunnelSettingsDidChange: ((LatestTunnelSettings) -> Void)?

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
