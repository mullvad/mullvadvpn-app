//
//  RelayFilterViewModel.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-06-09.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadREST
import MullvadSettings
import MullvadTypes

class RelayFilterViewModel {
    private var settings: LatestTunnelSettings
    private let relaysWithLocation: LocationRelays
    private let relaySelectorWrapper: RelaySelectorWrapper
    @Published var relayFilter: RelayFilter

    init(settings: LatestTunnelSettings, relaySelectorWrapper: RelaySelectorWrapper) {
        self.settings = settings
        self.relaySelectorWrapper = relaySelectorWrapper
        relaysWithLocation = if let cachedResponse = try? relaySelectorWrapper.relayCache.read().relays {
            LocationRelays(relays: cachedResponse.wireguard.relays, locations: cachedResponse.locations)
        } else {
            LocationRelays(relays: [], locations: [:])
        }

        self.relayFilter = if case let .only(filter) = settings.relayConstraints.filter {
            filter
        } else {
            RelayFilter()
        }
    }

    private var relays: [REST.ServerRelay] {
        relaysWithLocation.relays
    }

    var uniqueProviders: [String] {
        Set(relays.map { $0.provider }).caseInsensitiveSorted()
    }

    var ownedProviders: [String] {
        Set(relays.filter { $0.owned == true }.map { $0.provider }).caseInsensitiveSorted()
    }

    var rentedProviders: [String] {
        Set(relays.filter { $0.owned == false }.map { $0.provider }).caseInsensitiveSorted()
    }

    func addItemToFilter(_ item: RelayFilterDataSource.Item) {
        switch item {
        case .ownershipAny, .ownershipOwned, .ownershipRented:
            relayFilter.ownership = ownership(for: item) ?? .any
        case .allProviders:
            relayFilter.providers = .any
        case let .provider(name):
            switch relayFilter.providers {
            case .any:
                relayFilter.providers = .only([name])
            case var .only(providers):
                if !providers.contains(name) {
                    providers.append(name)
                    providers.caseInsensitiveSort()

                    if providers == availableProviders(for: relayFilter.ownership) {
                        relayFilter.providers = .any
                    } else {
                        relayFilter.providers = .only(providers)
                    }
                }
            }
        }
    }

    func removeItemFromFilter(_ item: RelayFilterDataSource.Item) {
        switch item {
        case .ownershipAny, .ownershipOwned, .ownershipRented:
            break
        case .allProviders:
            relayFilter.providers = .only([])
        case let .provider(name):
            switch relayFilter.providers {
            case .any:
                var providers = availableProviders(for: relayFilter.ownership)
                providers.removeAll { $0 == name }
                relayFilter.providers = .only(providers)
            case var .only(providers):
                providers.removeAll { $0 == name }
                relayFilter.providers = .only(providers)
            }
        }
    }

    func providerItem(for providerName: String?) -> RelayFilterDataSource.Item? {
        return .provider(providerName ?? "")
    }

    func availableProviders(for ownership: RelayFilter.Ownership) -> [String] {
        switch ownership {
        case .any:
            return uniqueProviders
        case .owned:
            return ownedProviders
        case .rented:
            return rentedProviders
        }
    }

    func ownership(for item: RelayFilterDataSource.Item?) -> RelayFilter.Ownership? {
        switch item {
        case .ownershipAny:
            return .any
        case .ownershipOwned:
            return .owned
        case .ownershipRented:
            return .rented
        default:
            return nil
        }
    }

    func ownershipItem(for ownership: RelayFilter.Ownership?) -> RelayFilterDataSource.Item? {
        switch ownership {
        case .any:
            return .ownershipAny
        case .owned:
            return .ownershipOwned
        case .rented:
            return .ownershipRented
        default:
            return nil
        }
    }

    func getFilteredRelays(_ relayFilter: RelayFilter) -> FilterDescriptor {
        settings.relayConstraints.filter = .only(relayFilter)
        do {
            let result = try relaySelectorWrapper.findCandidates(tunnelSettings: settings)
            return FilterDescriptor(relayFilterResult: result, settings: settings)
        } catch {
            return FilterDescriptor(
                relayFilterResult: RelaysCandidates(entryRelays: [], exitRelays: []),
                settings: settings
            )
        }
    }
}
