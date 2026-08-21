// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadSettings
import MullvadTypes

class MultihopTunnelSettingsViewModel: ObservableObject {
    enum ValidationError {
        case filters(state: MultihopState)
        case settings(state: MultihopState)
    }

    let tunnelManager: TunnelManager
    var tunnelObserver: TunnelObserver!

    var didFailValidation: ((ValidationError) -> Void)?

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

    func evaluate(setting: MultihopState) {
        if filtersWillBeOverridden(setting) {
            didFailValidation?(.filters(state: setting))
            return
        } else if stateIsIncompatible(setting) {
            didFailValidation?(.settings(state: setting))
            return
        }

        multihopState = setting
    }

    func filtersWillBeOverridden(_ state: MultihopState) -> Bool {
        let validator = MultihopValidator(
            tunnelSettings: tunnelManager.settings,
            relaySelector: tunnelManager.relaySelector
        )
        return validator.stateWillOverrideFilters(state)
    }

    func stateIsIncompatible(_ state: MultihopState) -> Bool {
        let validator = MultihopValidator(
            tunnelSettings: tunnelManager.settings,
            relaySelector: tunnelManager.relaySelector
        )
        return validator.stateIsIncompatible(state)
    }

    private func updateAutomaticRoutingStatus() {
        automaticRoutingIsActive =
            tunnelManager.settings.automaticMultihopIsEnabled
            && tunnelManager.tunnelStatus.state.isMultihop
    }
}
