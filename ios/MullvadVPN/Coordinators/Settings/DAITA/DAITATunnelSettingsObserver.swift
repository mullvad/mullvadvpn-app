//
//  DAITATunnelSettingsObserver.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-21.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

class DAITATunnelSettingsObserver: ObservableObject {
    private let tunnelManager: TunnelManager
    private var tunnelObserver: TunnelObserver?

    var value: DAITASettings {
        willSet(newValue) {
            guard newValue != self.value else { return }

            objectWillChange.send()
            tunnelManager.updateSettings([.daita(newValue)])
        }
    }

    init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager
        value = tunnelManager.settings.daita

        tunnelObserver = TunnelBlockObserver(didUpdateTunnelSettings: { [weak self] _, newSettings in
            self?.value = newSettings.daita
        })
    }
}
