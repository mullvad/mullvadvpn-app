//
//  RelayFilterViewModel.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-06-09.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadREST
import MullvadTypes

class RelayFilterViewModel {
    @Published var relays: [REST.ServerRelay]
    @Published var relayFilter: RelayFilter

    var uniqueProviders: [String] {
        return Set(relays.map { $0.provider }).caseInsensitiveSorted()
    }

    init(relays: [REST.ServerRelay], relayFilter: RelayFilter) {
        self.relays = relays
        self.relayFilter = relayFilter
    }

    func addItemToFilter(_ item: RelayFilterDataSource.Item) {
        switch item {
        case .ownershipAny, .ownershipOwned, .ownershipRented:
            relayFilter.ownership = getOwnership(for: item) ?? .any
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
                    relayFilter.providers = .only(providers)
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
                var providers = uniqueProviders
                providers.removeAll { $0 == name }
                relayFilter.providers = .only(providers)
            case var .only(providers):
                providers.removeAll { $0 == name }
                relayFilter.providers = .only(providers)
            }
        }
    }

    func getOwnership(for item: RelayFilterDataSource.Item?) -> RelayFilter.Ownership? {
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

    func getOwnershipItem(for ownership: RelayFilter.Ownership?) -> RelayFilterDataSource.Item? {
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

    func getProviderName(for item: RelayFilterDataSource.Item?) -> String? {
        switch item {
        case let .provider(name):
            return name
        default:
            return nil
        }
    }

    func getProviderItem(for providerName: String?) -> RelayFilterDataSource.Item? {
        return .provider(providerName ?? "")
    }
}
