//
//  MultihopTunnelSettingsViewModel.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-21.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings

class MultihopTunnelSettingsViewModel: TunnelSettingsObserver, ObservableObject {
    typealias TunnelSetting = MultihopState

    let tunnelManager: TunnelManager
    var tunnelObserver: TunnelObserver?

    var didFailValidation: ((MultihopState) -> Void)?

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
        if settingsAreIncompatible(setting) {
            didFailValidation?((setting))
            return
        }

        value = setting
    }

    private func settingsAreIncompatible(_ settings: MultihopState) -> Bool {
        var tunnelSettings = tunnelManager.settings
        tunnelSettings.tunnelMultihopState = settings

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
