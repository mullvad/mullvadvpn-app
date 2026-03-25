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

    var didFailDAITAValidation: ((DAITASettingsPromptItem) -> Void)?

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
            let promptItem = promptItem(from: error)

            didFailDAITAValidation?(promptItem)
            return
        }

        value = setting
    }
}

extension DAITATunnelSettingsViewModel {
    private func promptItem(from error: DAITASettingsCompatibilityError) -> DAITASettingsPromptItem {
        switch error {
        case .singlehop:
            .daitaSettingIncompatibleWithSinglehop
        case .multihop:
            .daitaSettingIncompatibleWithMultihop
        }
    }

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
