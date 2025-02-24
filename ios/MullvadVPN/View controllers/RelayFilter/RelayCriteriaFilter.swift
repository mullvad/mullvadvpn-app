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
    func matches(relays: LocationRelays, settings: LatestTunnelSettings) -> RelayFilterResult
}

struct RelayFilterResult {
    var entryRelays: LocationRelays?
    var exitRelays: LocationRelays
}

struct RelayFilterManager: RelayFilterable {
    func matches(relays: LocationRelays, settings: LatestTunnelSettings) -> RelayFilterResult {
        var entryRelays: LocationRelays? = settings.tunnelMultihopState.isEnabled ? relays : nil
        var exitRelays = relays

        if settings.daita.isDirectOnly {
            if var entry = entryRelays {
                entry.relays = relays.relays.filter { $0.daita == true }
                entryRelays = entry
            } else {
                exitRelays.relays = exitRelays.relays.filter { $0.daita == true }
            }
        }

        if case let .only(relayFilter) = settings.relayConstraints.filter {
            if var entry = entryRelays {
                entry.relays = entry.relays.filter { RelaySelector.relayMatchesFilter($0, filter: relayFilter) }
                entryRelays = entry
            }
            exitRelays.relays = exitRelays.relays.filter { RelaySelector.relayMatchesFilter($0, filter: relayFilter) }
        }

        return RelayFilterResult(entryRelays: entryRelays, exitRelays: exitRelays)
    }
}
