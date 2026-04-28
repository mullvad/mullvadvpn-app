//
//  MultihopTunnelSettingsViewModel.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-21.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings

class MultihopTunnelSettingsViewModel: ObservableObject {
    let tunnelManager: TunnelManager
    var tunnelObserver: TunnelObserver!

    @Published var automaticRoutingIsActive: Bool = false
    @Published var multihopState: MultihopState {
        willSet(newValue) {
            guard newValue != multihopState else { return }
            tunnelManager.updateSettings([.multihop(newValue)])
        }
    }

    required init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager
        multihopState = tunnelManager.settings.tunnelMultihopState

        tunnelObserver = TunnelBlockObserver(
            didUpdateTunnelStatus: { [weak self] _, _ in
                self?.updateAutomaticRoutingStatus()
            },
            didUpdateTunnelSettings: { [weak self] _, newSettings in
                self?.multihopState = newSettings.tunnelMultihopState
                self?.updateAutomaticRoutingStatus()
            }
        )

        self.tunnelManager.addObserver(tunnelObserver)
        updateAutomaticRoutingStatus()
    }

    private func updateAutomaticRoutingStatus() {
        automaticRoutingIsActive =
            tunnelManager.settings.automaticMultihopIsEnabled
            && tunnelManager.tunnelStatus.state.isMultihop
    }
}
