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
        // Check if multihop is enabled via settings
        let isMultihopEnabled = settings.tunnelMultihopState.isEnabled
        let isSmartRoutingEnabled = settings.daita.isAutomaticRouting

        /// Closure to check if there are enough relays available for multihoping
        let hasSufficientRelays: () -> Bool = {
            (relayFilterResult.entryRelays ?? []).count >= 1 &&
                relayFilterResult.exitRelays.count >= 1 &&
                numberOfServers > 1
        }

        if isMultihopEnabled {
            // Multihop mode requires at least one entry relay, one exit relay,
            // and more than one unique server.
            return hasSufficientRelays()
        } else if isSmartRoutingEnabled {
            // Smart Routing mode: Enabled only if there is NO daita server in the exit relays
            let isSmartRoutingNeeded = !relayFilterResult.exitRelays.contains { $0.relay.daita == true }
            return isSmartRoutingNeeded ? hasSufficientRelays() : true
        } else {
            // Single-hop mode: The filter is enabled if at least one available exit relay exists.
            return !relayFilterResult.exitRelays.isEmpty
        }
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
