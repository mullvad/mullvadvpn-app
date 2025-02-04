//
//  MultihopTunnelSettingsViewModel.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-21.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings

class MultihopTunnelSettingsViewModel: TunnelSettingsObserver, ObservableObject {
    typealias TunnelSetting = MultihopState

    let tunnelManager: TunnelManager
    var tunnelObserver: TunnelObserver?

    var value: MultihopState {
        willSet(newValue) {
            guard newValue != value else { return }

            objectWillChange.send()
            tunnelManager.updateSettings([.multihop(newValue)])
        }
    }

    required init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager
        value = tunnelManager.settings.tunnelMultihopState

        tunnelObserver = TunnelBlockObserver(didUpdateTunnelSettings: { [weak self] _, newSettings in
            self?.value = newSettings.tunnelMultihopState
        })
    }

    func evaluate(setting: MultihopState) {
        // No op.
    }
}

class MockMultihopTunnelSettingsViewModel: TunnelSettingsObservable {
    @Published var value: MultihopState

    init(multihopState: MultihopState = .off) {
        value = multihopState
    }

    func evaluate(setting: MullvadSettings.MultihopState) {}
}
