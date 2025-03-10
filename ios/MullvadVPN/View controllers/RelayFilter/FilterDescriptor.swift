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
        let isMultihopEnabled = settings.tunnelMultihopState.isEnabled || settings.daita.isAutomaticRouting
        return isMultihopEnabled ? (numberOfServers > 1) : (numberOfServers > 0)
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
        guard settings.daita.daitaState.isEnabled else {
            return ""
        }
        return NSLocalizedString(
            "RELAY_FILTER_BUTTON_DESCRIPTION",
            tableName: "RelayFilter",
            value: "When using DAITA, one provider with DAITA-enabled servers is required.",
            comment: ""
        )
    }

    init(relayFilterResult: RelayCandidates, settings: LatestTunnelSettings) {
        self.settings = settings
        self.relayFilterResult = relayFilterResult
    }

    private var numberOfServers: Int {
        Set(relayFilterResult.entryRelays ?? []).union(relayFilterResult.exitRelays).count
    }

    private func createTitleForAvailableServers() -> String {
        let displayNumber: (Int) -> String = { number in
            number >= 100 ? "99+" : "\(number)"
        }
        return String(format: "Show %@ servers", displayNumber(numberOfServers))
    }
}
