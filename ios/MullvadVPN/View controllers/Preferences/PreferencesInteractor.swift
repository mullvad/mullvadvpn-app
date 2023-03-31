//
//  PreferencesInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 31/10/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Refresh interval used for fetching current Wi-Fi network
private let wifiNetworkRefreshInterval = 10

final class PreferencesInteractor {
    private let tunnelManager: TunnelManager
    private var tunnelObserver: TunnelObserver?
    private var timer: DispatchSourceTimer?

    var tunnelSettingsDidChange: ((TunnelSettingsV2) -> Void)?

    init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager
        tunnelObserver = TunnelBlockObserver(didUpdateTunnelSettings: { [weak self] manager, newSettings in
            self?.tunnelSettingsDidChange?(newSettings)
        })
    }

    var tunnelSettings: TunnelSettingsV2 {
        return tunnelManager.settings
    }

    func updatTunnelSettings(
        dnsSettings: DNSSettings,
        trustedNetworkSettings: TrustedNetworkSettings,
        completion: (() -> Void)? = nil
    ) {
        tunnelManager.updateSettings(
            request: .init(
                dnsSettings: dnsSettings,
                trustedNetworkSettings: trustedNetworkSettings
            ),
            completionHandler: completion
        )
    }

    func startCurrentWifiNetworkRefresh(_ block: @escaping (ConnectedWifiNetwork?) -> Void) {
        timer?.cancel()

        timer = DispatchSource.makeTimerSource(queue: .main)

        timer?.setEventHandler {
            ConnectedWifiNetwork.fetchCurrent(block)
        }

        timer?.schedule(wallDeadline: .now(), repeating: .seconds(wifiNetworkRefreshInterval))
        timer?.activate()
    }

    func stopCurrentWifiNetworkRefresh() {
        timer?.cancel()
        timer = nil
    }
}
