//
//  DestinationDescriber.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2025-01-20.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
//  A source of truth for converting an exit relay destination (i.e., a relay or list) into a name

import MullvadREST
import MullvadSettings
import MullvadTypes

protocol DestinationDescribing {
    func describe(_ loc: UserSelectedRelays) -> String?
}

struct DestinationDescriber: DestinationDescribing {
    let relayCache: RelayCacheProtocol
    let customListRepository: CustomListRepositoryProtocol

    public init(
        relayCache: RelayCacheProtocol,
        customListRepository: CustomListRepositoryProtocol
    ) {
        self.relayCache = relayCache
        self.customListRepository = customListRepository
    }

    func describe(_ loc: UserSelectedRelays) -> String? {
        if let listName = (loc.customListSelection.flatMap {
            customListRepository.fetch(by: $0.listId)?.name
        }) {
            return listName
        }
        guard
            let location = loc.locations.first,
            let cachedRelays = try? relayCache.read().relays
        else { return nil }
        let locatedRelays = RelayWithLocation.locateRelays(
            relays: cachedRelays.wireguard.relays,
            locations: cachedRelays.locations
        )

        return locatedRelays.first { $0.matches(location: location) }?.serverLocation.country
    }
}
