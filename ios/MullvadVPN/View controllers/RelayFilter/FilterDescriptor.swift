//
//  FilterDescriptor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-02-25.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import MullvadREST
import MullvadSettings
struct FilterDescriptor {
    let relayFilterResult: RelaysCandidates
    let settings: LatestTunnelSettings

    var isEnabled: Bool {
        let exitCount = relayFilterResult.exitRelays.count
        let entryCount = relayFilterResult.entryRelays?.count ?? 0
        let totalcount = exitCount + entryCount
        let isMultihopEnabled = settings.tunnelMultihopState.isEnabled
        return (isMultihopEnabled && totalcount > 1) || (!isMultihopEnabled && totalcount > 0)
    }

    var title: String {
        let exitCount = relayFilterResult.exitRelays.count
        let entryCount = relayFilterResult.entryRelays?.count ?? 0
        guard isEnabled else {
            return NSLocalizedString(
                "RELAY_FILTER_BUTTON_TITLE",
                tableName: "RelayFilter",
                value: "No matching servers",
                comment: ""
            )
        }
        return createTitleForAvailableServers(
            entryCount: entryCount,
            exitCount: exitCount,
            isMultihopEnabled: settings.tunnelMultihopState.isEnabled,
            isDirectOnly: settings.daita.isDirectOnly
        )
    }

    var description: String {
        guard settings.daita.isDirectOnly else {
            return ""
        }
        return NSLocalizedString(
            "RELAY_FILTER_BUTTON_DESCRIPTION",
            tableName: "RelayFilter",
            value: "Direct only DAITA is enabled, affecting your filters.",
            comment: ""
        )
    }

    init(relayFilterResult: RelaysCandidates, settings: LatestTunnelSettings) {
        self.settings = settings
        self.relayFilterResult = relayFilterResult
    }

    private func createTitleForAvailableServers(
        entryCount: Int,
        exitCount: Int,
        isMultihopEnabled: Bool,
        isDirectOnly: Bool
    ) -> String {
        let displayNumber: (Int) -> String = { number in
            number > 100 ? "99+" : "\(number)"
        }

        if isMultihopEnabled && isDirectOnly {
            return String(
                format: "Show %@ entry & %@ exit servers",
                displayNumber(entryCount),
                displayNumber(exitCount)
            )
        }
        return String(format: "Show %@ servers", displayNumber(exitCount))
    }
}
