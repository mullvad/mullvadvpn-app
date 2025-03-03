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
    @Published var relayFilter: RelayFilter

    private var settings: LatestTunnelSettings
    private let relaySelectorWrapper: RelaySelectorWrapper
    private let relaysWithLocation: LocationRelays
    private var providerStatusCache: [String: Bool] = [:]

    init(settings: LatestTunnelSettings, relaySelectorWrapper: RelaySelectorWrapper) {
        self.settings = settings
        self.relaySelectorWrapper = relaySelectorWrapper

        // Directly setting relaysWithLocation in constructor
        if let cachedResponse = try? relaySelectorWrapper.relayCache.read().relays {
            self.relaysWithLocation = LocationRelays(
                relays: cachedResponse.wireguard.relays,
                locations: cachedResponse.locations
            )
        } else {
            self.relaysWithLocation = LocationRelays(relays: [], locations: [:])
        }

        self.relayFilter = if case let .only(filter) = settings.relayConstraints.filter {
            filter
        } else {
            RelayFilter()
        }
    }

    // MARK: - Computed Properties

    private var relays: [REST.ServerRelay] { relaysWithLocation.relays }

    var uniqueProviders: [String] {
        extractProviders(from: relays)
    }

    var ownedProviders: [String] {
        extractProviders(from: relays.filter { $0.owned == true })
    }

    var rentedProviders: [String] {
        extractProviders(from: relays.filter { $0.owned == false })
    }

    // MARK: - public Methods

    func toggleItem(_ item: RelayFilterDataSource.Item) {
        switch item.type {
        case .ownershipAny, .ownershipOwned, .ownershipRented:
            relayFilter.ownership = ownership(for: item) ?? .any
        case .allProviders:
            relayFilter.providers = relayFilter.providers == .any ? .only([]) : .any
        case let .provider(name):
            toggleProvider(name)
        }
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

    func ownership(for item: RelayFilterDataSource.Item) -> RelayFilter.Ownership? {
        switch item.type {
        case .ownershipAny: return .any
        case .ownershipOwned: return .owned
        case .ownershipRented: return .rented
        default: return nil
        }
    }

    func ownershipItem(for ownership: RelayFilter.Ownership) -> RelayFilterDataSource.Item {
        switch ownership {
        case .any:
            RelayFilterDataSource.Item.ownerships[0]
        case .owned:
            RelayFilterDataSource.Item.ownerships[1]
        case .rented:
            RelayFilterDataSource.Item.ownerships[2]
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

    // MARK: - private Methods

    private func toggleProvider(_ name: String) {
        switch relayFilter.providers {
        case .any:
            // If currently "any", switch to only the selected provider
            var providers = availableProviders(for: relayFilter.ownership)
            providers.removeAll { $0 == name }
            relayFilter.providers = .only(providers)
        case var .only(providers):
            if providers.contains(name) {
                // If provider exists, remove it
                providers.removeAll { $0 == name }
            } else {
                // Otherwise, add it
                providers.append(name)
                providers.caseInsensitiveSort()
            }

            // If all available providers are selected, switch back to "any"
            relayFilter.providers = providers.isEmpty
                ? .only([])
                : (providers == availableProviders(for: relayFilter.ownership) ? .any : .only(providers))
        }
    }

    // MARK: - Private Helper Methods

    private func extractProviders(from relays: [REST.ServerRelay]) -> [String] {
        Set(relays.map { $0.provider }).caseInsensitiveSorted()
    }

    func providerItem(for providerName: String) -> RelayFilterDataSource.Item {
        return RelayFilterDataSource.Item.provider(name: providerName, isEnabled: isProviderEnabled(for: providerName))
    }

    private func isProviderEnabled(for providerName: String) -> Bool {
        let isFilterPossible: Bool = getFilteredRelays(relayFilter).isEnabled
        // Check if the current filter allows any provider
        if relayFilter.providers == .any {
            return isFilterPossible
        }

        // If provider is already selected, use the main filter result
        if case let .only(providers) = relayFilter.providers, providers.contains(providerName) {
            return isFilterPossible
        }

        // Check cache before computing
        if let cachedStatus = providerStatusCache[providerName] {
            return cachedStatus
        }

        // Compute and cache the provider's status
        let computedStatus = getFilteredRelays(
            RelayFilter(ownership: .any, providers: .only([providerName]))
        ).isEnabled

        providerStatusCache[providerName] = computedStatus
        return computedStatus
    }
}
