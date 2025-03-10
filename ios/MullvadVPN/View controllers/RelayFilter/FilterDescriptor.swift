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
    let relayFilterResult: RelayCandidates
    let settings: LatestTunnelSettings

    var isEnabled: Bool {
        let exitCount = relayFilterResult.exitRelays.count
        let entryCount = relayFilterResult.entryRelays?.count ?? 0
        let isMultihopEnabled = settings.tunnelMultihopState.isEnabled || settings.daita.isAutomaticRouting
        return (isMultihopEnabled && entryCount > 1 && exitCount > 1) || (!isMultihopEnabled && exitCount > 0)
    }

    var title: String {
        guard isEnabled else {
            return NSLocalizedString(
                "RELAY_FILTER_BUTTON_TITLE",
                tableName: "RelayFilter",
                value: "No matching servers",
                comment: ""
            )
        }
        return createTitleForAvailableServers()
    }

    var description: String {
        guard settings.daita.isDirectOnly else {
            return settings.daita.daitaState.isEnabled
                ? NSLocalizedString(
                    "RELAY_FILTER_BUTTON_DESCRIPTION",
                    tableName: "RelayFilter",
                    value: "DAITA is enabled, affecting your filters.",
                    comment: ""
                )
                : ""
        }
        return NSLocalizedString(
            "RELAY_FILTER_BUTTON_DESCRIPTION",
            tableName: "RelayFilter",
            value: "Direct only DAITA is enabled, affecting your filters.",
            comment: ""
        )
    }

    init(relayFilterResult: RelayCandidates, settings: LatestTunnelSettings) {
        self.settings = settings
        self.relayFilterResult = relayFilterResult
    }

    private func createTitleForAvailableServers() -> String {
        let displayNumber: (Int) -> String = { number in
            number >= 100 ? "99+" : "\(number)"
        }

        let numberOfRelays = Set(relayFilterResult.entryRelays ?? []).union(relayFilterResult.exitRelays).count
        return String(format: "Show %@ servers", displayNumber(numberOfRelays))
    }
}
