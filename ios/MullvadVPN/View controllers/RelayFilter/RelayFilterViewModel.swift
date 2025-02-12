//
//  RelayFilterViewModel.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-06-09.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadREST
import MullvadTypes

class RelayFilterViewModel {
    @Published var relays: [REST.ServerRelay]
    @Published var relayFilter: RelayFilter

    var uniqueProviders: [String] {
        Set(relays.map { $0.provider }).caseInsensitiveSorted()
    }

    var ownedProviders: [String] {
        Set(relays.filter { $0.owned == true }.map { $0.provider }).caseInsensitiveSorted()
    }

    var rentedProviders: [String] {
        Set(relays.filter { $0.owned == false }.map { $0.provider }).caseInsensitiveSorted()
    }

    init(relays: [REST.ServerRelay], relayFilter: RelayFilter) {
        self.relays = relays
        self.relayFilter = relayFilter
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

    func providerName(for item: RelayFilterDataSource.Item?) -> String? {
        switch item {
        case let .provider(name):
            return name
        default:
            return nil
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
}

extension RelayFilterViewModel {
    func getFilteredRelays(with filter: RelayFilter) -> [String] {
        let relays = { relays in
            switch (filter.ownership, filter.providers) {
            case (.any, .any):
                return relays
            case let (.owned, .only(providers)), let (.rented, .only(providers)):
                let isOwned = filter.ownership == .owned
                return relays.filter { $0.owned == isOwned && providers.contains($0.provider) }
            case (.owned, .any), (.rented, .any):
                let isOwned = filter.ownership == .owned
                return relays.filter { $0.owned == isOwned }
            case let (.any, .only(providers)):
                return relays.filter { providers.contains($0.provider) }
            }
        }(relays)

        return relays.map { $0.hostname }
    }
}
