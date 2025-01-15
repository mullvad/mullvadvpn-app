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
    func describe(_ destination: UserSelectedRelays) -> String?
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

    private func customListDescription(_ destination: UserSelectedRelays) -> String? {
        // We only return a description for the list if the user has selected the
        // entire list. If they have only selected relays/locations from it,
        // we show those as if they selected them from elsewhere.
        guard
            let customListSelection = destination.customListSelection,
            customListSelection.isList,
            let customList = customListRepository.fetch(by: customListSelection.listId)
        else { return nil }
        return customList.name
    }

    private func describeRelayLocation(
        _ locationSpec: RelayLocation,
        usingRelayWithLocation serverLocation: Location
    ) -> String {
        switch locationSpec {
        case .country: serverLocation.country
        case .city: serverLocation.city
        case let .hostname(_, _, hostname):
            "\(serverLocation.city) (\(hostname))"
        }
    }

    private func relayDescription(_ destination: UserSelectedRelays) -> String? {
        guard
            let location = destination.locations.first,
            let cachedRelays = try? relayCache.read().relays
        else { return nil }
        let locatedRelays = RelayWithLocation.locateRelays(
            relays: cachedRelays.wireguard.relays,
            locations: cachedRelays.locations
        )

        guard let matchingRelay = (locatedRelays.first { $0.matches(location: location)
        }) else { return nil }

        return describeRelayLocation(location, usingRelayWithLocation: matchingRelay.serverLocation)
    }

    func describe(_ destination: UserSelectedRelays) -> String? {
        customListDescription(destination) ?? relayDescription(destination)
    }
}
