//
//  PreferencesInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 31/10/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

final class PreferencesInteractor {
    private let tunnelManager: TunnelManager
    private var tunnelObserver: TunnelObserver?

    var tunnelSettingsDidChange: ((TunnelSettingsV2) -> Void)?

    init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager
        tunnelObserver =
            TunnelBlockObserver(didUpdateTunnelSettings: { [weak self] manager, newSettings in
                self?.tunnelSettingsDidChange?(newSettings)
            })
    }

    var tunnelSettings: TunnelSettingsV2 {
        return tunnelManager.settings
    }

    func setDNSSettings(_ newDNSSettings: DNSSettings, completion: (() -> Void)? = nil) {
        tunnelManager.setDNSSettings(newDNSSettings, completionHandler: completion)
    }

    func setPort(_ port: UInt16?, completion: (() -> Void)? = nil) {
        var relayConstraints = tunnelManager.settings.relayConstraints

        if let port = port {
            relayConstraints.port = .only(port)
        } else {
            relayConstraints.port = .any
        }

        tunnelManager.setRelayConstraints(relayConstraints, completionHandler: completion)
    }
}
