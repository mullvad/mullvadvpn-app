//
//  DAITATunnelSettingsViewModel.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-21.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadSettings

class DAITATunnelSettingsViewModel: TunnelSettingsObserver {
    typealias TunnelSetting = DAITASettings

    let tunnelManager: TunnelManager
    var tunnelObserver: TunnelObserver?

    var isAutomaticRoutingActive: Bool

    var didFailDAITAValidation: (() -> Void)?

    @Published var value: DAITASettings {
        willSet {
            guard newValue != value else { return }
            tunnelManager.updateSettings([.daita(newValue)])
        }
    }

    required init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager
        value = tunnelManager.settings.daita

        var isAutomaticRoutingActive: Bool {
            tunnelManager.tunnelStatus.state.isMultihop && tunnelManager.settings.tunnelMultihopState.isWhenNeeded
        }
        self.isAutomaticRoutingActive = isAutomaticRoutingActive

        let tunnelObserver = TunnelBlockObserver(didUpdateTunnelStatus: { [weak self] _, _ in
            if isAutomaticRoutingActive != self?.isAutomaticRoutingActive {
                self?.isAutomaticRoutingActive = isAutomaticRoutingActive
                self?.objectWillChange.send()
            }
        })
        self.tunnelObserver = tunnelObserver

        tunnelManager.addObserver(tunnelObserver)
    }

    func evaluate(setting: DAITASettings) {
        if let error = evaluateDaitaSettingsCompatibility(setting) {
            didFailDAITAValidation?()
            return
        }

        value = setting
    }
}

extension DAITATunnelSettingsViewModel {
    private func evaluateDaitaSettingsCompatibility(_ settings: DAITASettings) -> DAITASettingsCompatibilityError? {
        guard settings.isEnabled else { return nil }

        var tunnelSettings = tunnelManager.settings
        tunnelSettings.daita = settings

        let relays = try? tunnelManager.selectRelays(tunnelSettings: tunnelSettings)

        return if relays == nil {
            tunnelSettings.tunnelMultihopState.isAlways ? .multihop : .singlehop
        } else {
            nil
        }
    }
}

class MockDAITATunnelSettingsViewModel: TunnelSettingsObservable {
    @Published var value: DAITASettings

    init(daitaSettings: DAITASettings = DAITASettings()) {
        value = daitaSettings
    }

    func evaluate(setting: MullvadSettings.DAITASettings) {}
}
