//
//  MultihopValidator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-05-21.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

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
