//
//  MultihopTunnelSettingsViewModel.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-21.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings

class MultihopTunnelSettingsViewModel: TunnelSettingsObserver {
    typealias TunnelSetting = MultihopState

    let tunnelManager: TunnelManager
    var tunnelObserver: TunnelObserver?

    var didFailValidation: ((MultihopState) -> Void)?

    var value: MultihopState {
        willSet(newValue) {
            guard newValue != value else { return }
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
        if stateIsIncompatible(setting) {
            didFailValidation?((setting))
            return
        }

        value = setting
    }

    func stateIsIncompatible(_ state: MultihopState) -> Bool {
        var tunnelSettings = tunnelManager.settings
        tunnelSettings.tunnelMultihopState = state

        if !tunnelSettings.automaticMultihopIsEnabled {
            let relays = try? tunnelManager.selectRelays(tunnelSettings: tunnelSettings)
            return relays == nil
        }

        return false
    }
}

class MockMultihopTunnelSettingsViewModel: TunnelSettingsObservable {
    @Published var value: MultihopState

    init(multihopState: MultihopState = .never) {
        value = multihopState
    }

    func evaluate(setting: MullvadSettings.MultihopState) {}
}
