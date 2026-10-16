// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
        guard evaluateDaitaSettingsCompatibility(setting) == nil else {
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
