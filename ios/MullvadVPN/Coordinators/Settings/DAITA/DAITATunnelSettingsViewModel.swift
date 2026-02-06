//
//  DAITATunnelSettingsViewModel.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-21.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadSettings

class DAITATunnelSettingsViewModel: TunnelSettingsObserver, ObservableObject {
    typealias TunnelSetting = DAITASettings

    let tunnelManager: TunnelManager
    var tunnelObserver: TunnelObserver?

    var isAutomaticRoutingActive: Bool

    var didFailDAITAValidation: (((item: DAITASettingsPromptItem, setting: DAITASettings)) -> Void)?

    var value: DAITASettings {
        willSet {
            guard newValue != value else { return }

            objectWillChange.send()
            tunnelManager.updateSettings([.daita(newValue)])
        }
    }

    required init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager
        value = tunnelManager.settings.daita

        var isAutomaticRoutingActive: Bool {
            tunnelManager.tunnelStatus.state.isMultihop && !tunnelManager.settings.tunnelMultihopState.isEnabled
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
            let promptItem = promptItem(from: error, setting: setting)

            didFailDAITAValidation?((item: promptItem, setting: setting))
            return
        }

        value = setting
    }
}

extension DAITATunnelSettingsViewModel {
    private func promptItem(
        from error: DAITASettingsCompatibilityError,
        setting: DAITASettings
    ) -> DAITASettingsPromptItem {
        let promptItemSetting: DAITASettingsPromptItem.Setting =
            if setting.daitaState != value.daitaState {
                .daita
            } else {
                .directOnly
            }

        var promptItem: DAITASettingsPromptItem

        switch error {
        case .singlehop:
            promptItem = .daitaSettingIncompatibleWithSinglehop(promptItemSetting)
        case .multihop:
            promptItem = .daitaSettingIncompatibleWithMultihop(promptItemSetting)
        }

        return promptItem
    }

    private func evaluateDaitaSettingsCompatibility(_ settings: DAITASettings) -> DAITASettingsCompatibilityError? {
        guard settings.daitaState.isEnabled else { return nil }

        var tunnelSettings = tunnelManager.settings
        tunnelSettings.daita = settings

        var compatibilityError: DAITASettingsCompatibilityError?

        if settings.isDirectOnly {
            let relays = try? tunnelManager.selectRelays(tunnelSettings: tunnelSettings)

            // Even if the reason for not finding any relays is not DAITA specific, if both DAITA and Direct
            // only are enabled, we should return a DAITA related error since the current settings would have
            // resulted in the relay selector not being able to select a DAITA relay anyway.
            if relays == nil {
                compatibilityError = tunnelSettings.tunnelMultihopState.isEnabled ? .multihop : .singlehop
            }
        }

        return compatibilityError
    }
}

class MockDAITATunnelSettingsViewModel: TunnelSettingsObservable {
    @Published var value: DAITASettings

    init(daitaSettings: DAITASettings = DAITASettings()) {
        value = daitaSettings
    }

    func evaluate(setting: MullvadSettings.DAITASettings) {}
}
