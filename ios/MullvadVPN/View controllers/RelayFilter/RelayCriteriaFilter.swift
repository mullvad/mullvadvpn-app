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
    func matches(relays: LocationRelays, criteria: RelayFilterCriteria) -> LocationRelays
}

struct RelayFilterCriteria {
    let isDirectOnly: Bool
    let filterConstraints: RelayFilter
}

struct RelayFilterManager: RelayFilterable {
    func matches(relays: LocationRelays, criteria: RelayFilterCriteria) -> LocationRelays {
        var relaysWithLocation = relays
        relaysWithLocation.relays = if criteria.isDirectOnly {
            relaysWithLocation.relays.filter { relay in
                RelaySelector.relayMatchesFilter(relay, filter: criteria.filterConstraints) && relay.daita == true
            }
        } else {
            relaysWithLocation.relays.filter { relay in
                RelaySelector.relayMatchesFilter(relay, filter: criteria.filterConstraints)
            }
        }
        return relaysWithLocation
    }
}
