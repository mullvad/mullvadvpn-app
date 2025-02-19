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

enum RelayFilterError: Error {
    case notFound
}

protocol RelayFilterable {
    func matches(relays: [REST.ServerRelay], criteria: RelayFilterCriteria) throws(RelayFilterError)
        -> [REST.ServerRelay]
}

struct RelayFilterCriteria {
    let daitaSettings: DAITASettings
    let filterConstraints: RelayFilter
}

struct RelayFilterSelector: RelayFilterable {
    func matches(
        relays: [REST.ServerRelay],
        criteria: RelayFilterCriteria
    ) throws(RelayFilterError) -> [REST.ServerRelay] {
        var relaysWithLocation = LocationRelays(
            relays: relays.wireguard.relays,
            locations: relays.locations
        )
        relaysWithLocation.relays = relaysWithLocation.relays.filter { relay in
            RelaySelector.relayMatchesFilter(relay, filter: filter)
        }
        throw RelayFilterError.notFound
    }
}
