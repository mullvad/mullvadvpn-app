//
//  RelayCriteriaFilter.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-02-19.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import MullvadREST
import MullvadSettings
import MullvadTypes

protocol RelayFilterable {
    func matches(relays: LocationRelays, criteria: FilterCriteria) -> RelayFilterResult
}

struct RelayFilterResult {
    var entryRelays: LocationRelays?
    var exitRelays: LocationRelays
}

struct FilterCriteria {
    var settings: LatestTunnelSettings
    var relayFilter = RelayFilter()

    var isMultihoping: Bool {
        settings.tunnelMultihopState.isEnabled
    }

    var isDirectOnly: Bool {
        settings.daita.isDirectOnly
    }
}

struct RelayFilterManager: RelayFilterable {
    func matches(relays: LocationRelays, criteria: FilterCriteria) -> RelayFilterResult {
        var entryRelays: LocationRelays? = criteria.isMultihoping ? relays : nil
        var exitRelays = relays

        if criteria.isDirectOnly {
            if var entry = entryRelays {
                entry.relays = relays.relays.filter { $0.daita == true }
                entryRelays = entry
            } else {
                exitRelays.relays = exitRelays.relays.filter { $0.daita == true }
            }
        }

        if var entry = entryRelays {
            entry.relays = entry.relays.filter { RelaySelector.relayMatchesFilter($0, filter: criteria.relayFilter) }
            entryRelays = entry
        }
        exitRelays.relays = exitRelays.relays
            .filter { RelaySelector.relayMatchesFilter($0, filter: criteria.relayFilter) }

        return RelayFilterResult(entryRelays: entryRelays, exitRelays: exitRelays)
    }
}
