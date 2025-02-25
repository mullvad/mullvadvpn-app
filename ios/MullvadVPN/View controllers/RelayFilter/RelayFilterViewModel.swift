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
    private var relaysWithLocation: LocationRelays
    private let relayFilterManager: RelayFilterable

    var onNewSettings: ((LatestTunnelSettings) -> Void)?
    var onNewRelays: ((LocationRelays) -> Void)?
    @Published var relayFilter: RelayFilter

    init(settings: LatestTunnelSettings, relaysWithLocation: LocationRelays, relayFilterManager: RelayFilterable) {
        self.settings = settings
        self.relaysWithLocation = relaysWithLocation
        self.relayFilterManager = relayFilterManager
        self.relayFilter = if case let .only(filter) = settings.relayConstraints.filter {
            filter
        } else {
            RelayFilter()
        }
        self.onNewRelays = { [weak self] newRelays in
            self?.relaysWithLocation = newRelays
        }

        self.onNewSettings = { [weak self] newSettings in
            self?.settings = newSettings
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
        let relayFilterResult = relayFilterManager.matches(
            relays: relaysWithLocation,
            criteria: FilterCriteria(
                settings: settings,
                relayFilter: relayFilter
            )
        )
        return FilterDescriptor(relayFilterResult: relayFilterResult, settings: settings)
    }
}
