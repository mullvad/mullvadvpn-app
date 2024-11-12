//
//  MultihopTunnelSettingsObserver.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-21.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

class MultihopTunnelSettingsObserver: ObservableObject {
    private let tunnelManager: TunnelManager
    private var tunnelObserver: TunnelObserver?

    var value: MultihopState {
        willSet(newValue) {
            guard newValue != value else { return }

            objectWillChange.send()
            tunnelManager.updateSettings([.multihop(newValue)])
        }
    }

    init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager
        value = tunnelManager.settings.tunnelMultihopState

        tunnelObserver = TunnelBlockObserver(didUpdateTunnelSettings: { [weak self] _, newSettings in
            self?.value = newSettings.tunnelMultihopState
        })
    }
}
