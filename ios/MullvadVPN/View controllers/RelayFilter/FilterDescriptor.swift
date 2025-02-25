//
//  FilterDescriptor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-02-25.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import MullvadSettings
struct FilterDescriptor {
    let relayFilterResult: RelayFilterResult
    let settings: LatestTunnelSettings

    var isEnabled: Bool {
        let exitCount = relayFilterResult.exitRelays.relays.count
        let entryCount = relayFilterResult.entryRelays?.relays.count ?? 0
        let isMultihopEnabled = settings.tunnelMultihopState.isEnabled
        return (isMultihopEnabled && (entryCount > 0 || exitCount > 0)) || (!isMultihopEnabled && exitCount > 0)
    }

    var title: String {
        let exitCount = relayFilterResult.exitRelays.relays.count
        let entryCount = relayFilterResult.entryRelays?.relays.count ?? 0
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
        guard isEnabled else {
            return NSLocalizedString(
                "RELAY_FILTER_BUTTON_DESCRIPTION",
                tableName: "RelayFilter",
                value: "No matching servers found. Please try changing your filters.",
                comment: ""
            )
        }
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

    init(relayFilterResult: RelayFilterResult, settings: LatestTunnelSettings) {
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
