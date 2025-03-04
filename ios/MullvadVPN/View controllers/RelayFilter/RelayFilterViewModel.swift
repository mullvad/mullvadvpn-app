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

final class RelayFilterViewModel: @unchecked Sendable {
    @Published var relayFilter: RelayFilter

    private var settings: LatestTunnelSettings
    private let relaySelectorWrapper: RelaySelectorWrapper
    private let relaysWithLocation: LocationRelays
    private var providerStatusCache = ProviderStatusCache()

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
                relayFilterResult: RelayCandidates(entryRelays: [], exitRelays: []),
                settings: settings
            )
        }
    }

    func providerItem(for providerName: String) async -> RelayFilterDataSource.Item {
        let isEnabled = await isProviderEnabled(for: providerName)
        return RelayFilterDataSource.Item.provider(name: providerName, isEnabled: isEnabled)
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
            }

            // If all available providers are selected, switch back to "any"
            relayFilter.providers = providers.isEmpty
                ? .only([])
                : (providers == availableProviders(for: relayFilter.ownership) ? .any : .only(providers))
        }
    }

    private func extractProviders(from relays: [REST.ServerRelay]) -> [String] {
        Set(relays.map { $0.provider }).caseInsensitiveSorted()
    }

    private func isProviderEnabled(for providerName: String) async -> Bool {
        let relayFilterSnapshot = relayFilter
        let isFilterPossible = getFilteredRelays(relayFilterSnapshot).isEnabled

        if relayFilterSnapshot.providers == .any {
            return isFilterPossible
        }

        if case let .only(providers) = relayFilterSnapshot.providers, providers.contains(providerName) {
            return isFilterPossible
        }

        // Check cache safely
        if let cachedStatus = await providerStatusCache.get(providerName) {
            return cachedStatus
        }

        // Run computation asynchronously in a safe way
        let numberOfCompatibleRelays = getFilteredRelays(
            RelayFilter(ownership: .any, providers: .only([providerName]))
        ).isEnabled

        // Store result safely
        await providerStatusCache.set(providerName, value: numberOfCompatibleRelays)

        return numberOfCompatibleRelays
    }
}

private actor ProviderStatusCache {
    private var cache: [String: Bool] = [:]

    func get(_ providerName: String) -> Bool? {
        return cache[providerName]
    }

    func set(_ providerName: String, value: Bool) {
        cache[providerName] = value
    }
}
