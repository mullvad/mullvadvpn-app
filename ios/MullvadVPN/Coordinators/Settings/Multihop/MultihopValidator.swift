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
import MullvadTypes

struct MultihopValidator {
    private let tunnelSettings: LatestTunnelSettings
    private let relaySelector: RelaySelectorProtocol

    private var hasEntryFilter: Bool {
        tunnelSettings.relayConstraints.entryFilter.value?.isActive ?? false
    }

    init(tunnelSettings: LatestTunnelSettings, relaySelector: RelaySelectorProtocol) {
        self.tunnelSettings = tunnelSettings
        self.relaySelector = relaySelector
    }

    func stateWillOverrideFilters(_ state: MultihopState) -> Bool {
        // No need to warn if we're already overriding filters.
        guard !tunnelSettings.automaticMultihopIsEnabled else {
            return false
        }

        var tunnelSettings = tunnelSettings
        tunnelSettings.tunnelMultihopState = state

        return tunnelSettings.automaticMultihopIsEnabled && hasEntryFilter
    }

    func locationWillOverrideFilters(_ node: LocationNode, context: MultihopContext) -> Bool {
        // No need to warn if we're already overriding filters.
        guard !tunnelSettings.automaticMultihopIsEnabled else {
            return false
        }

        return context == .entry
            && node is AutomaticLocationNode
            && hasEntryFilter
    }

    func stateIsIncompatible(_ state: MultihopState) -> Bool {
        var tunnelSettings = tunnelSettings
        tunnelSettings.tunnelMultihopState = state

        if !tunnelSettings.automaticMultihopIsEnabled {
            let relays = try? relaySelector.selectRelays(tunnelSettings: tunnelSettings, connectionAttemptCount: 0)
            return relays == nil
        }

        return false
    }
}
