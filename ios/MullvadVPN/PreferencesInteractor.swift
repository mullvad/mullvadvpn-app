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

    var dnsSettingsDidChange: ((DNSSettings) -> Void)?

    init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager
        tunnelObserver =
            TunnelBlockObserver(didUpdateTunnelSettings: { [weak self] manager, newSettings in
                self?.dnsSettingsDidChange?(newSettings.dnsSettings)
            })
    }

    var dnsSettings: DNSSettings {
        return tunnelManager.settings.dnsSettings
    }

    func setDNSSettings(_ newDNSSettings: DNSSettings, completion: (() -> Void)? = nil) {
        tunnelManager.setDNSSettings(newDNSSettings, completionHandler: completion)
    }
}
