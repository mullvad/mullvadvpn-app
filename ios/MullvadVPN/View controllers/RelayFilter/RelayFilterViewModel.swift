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

final class RelayFilterViewModel {
    @Published var relayFilter: RelayFilter

    private var settings: LatestTunnelSettings
    private let relaySelectorWrapper: RelaySelectorWrapper
    private let relaysWithLocation: LocationRelays
    private var relayCandidatesForAny: RelayCandidates

    init(settings: LatestTunnelSettings, relaySelectorWrapper: RelaySelectorWrapper) {
        self.settings = settings
        self.relaySelectorWrapper = relaySelectorWrapper
        self.relayFilter = settings.relayConstraints.filter.value ?? RelayFilter()

        // Retrieve all available relays that satisfy the `any` constraint.
        // This constraint ensures that the selected relays are associated with the current tunnel settings
        // and serve as the primary source of truth for subsequent filtering operations.
        // Further filtering will be applied based on specific criteria such as `ownership` or `provider`.
        var copy = settings
        copy.relayConstraints.filter = .any
        if let relayCandidatesForAny = try? relaySelectorWrapper.findCandidates(tunnelSettings: copy) {
            self.relayCandidatesForAny = relayCandidatesForAny
        } else {
            self.relayCandidatesForAny = RelayCandidates(entryRelays: nil, exitRelays: [])
        }

        // Directly setting relaysWithLocation in constructor
        if let cachedResponse = try? relaySelectorWrapper.relayCache.read().relays {
            self.relaysWithLocation = LocationRelays(
                relays: cachedResponse.wireguard.relays,
                locations: cachedResponse.locations
            )
        } else {
            self.relaysWithLocation = LocationRelays(relays: [], locations: [:])
        }
    }

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
        case .provider:
            toggleProvider(item.name)
        }
    }

    func availableProviders(for ownership: RelayFilter.Ownership) -> [RelayFilterDataSource.Item] {
        providers(for: ownership)
            .map {
                providerItem(for: $0)
            }.sorted()
    }

    func ownership(for item: RelayFilterDataSource.Item) -> RelayFilter.Ownership? {
        let ownershipMapping: [RelayFilterDataSource.Item.ItemType: RelayFilter.Ownership] = [
            .ownershipAny: .any,
            .ownershipOwned: .owned,
            .ownershipRented: .rented,
        ]

        return ownershipMapping[item.type]
    }

    func ownershipItem(for ownership: RelayFilter.Ownership) -> RelayFilterDataSource.Item? {
        let ownershipMapping: [RelayFilter.Ownership: RelayFilterDataSource.Item.ItemType] = [
            .any: .ownershipAny,
            .owned: .ownershipOwned,
            .rented: .ownershipRented,
        ]

        return RelayFilterDataSource.Item.ownerships.first { $0.type == ownershipMapping[ownership] }
    }

    func providerItem(for providerName: String) -> RelayFilterDataSource.Item {
        let isDaitaEnabled = settings.daita.daitaState.isEnabled
        let isProviderEnabled = isProviderEnabled(for: providerName)
        let isFilterable = getFilteredRelays(relayFilter).isEnabled

        let statusText = isDaitaEnabled
            ? NSLocalizedString(
                "ENABLED_LABEL",
                tableName: "RelayFilter",
                value: "enabled",
                comment: ""
            )
            : ""

        return RelayFilterDataSource.Item(
            name: providerName,
            description: isDaitaEnabled && isProviderEnabled
                ? String(
                    format: NSLocalizedString(
                        "RELAY_FILTER_PROVIDER_DESCRIPTION_FORMAT_LABEL",
                        tableName: "RelayFilter",
                        value: "DAITA-%@",
                        comment: "Format for DAITA provider description"
                    ), statusText
                )
                : "",
            type: .provider,
            // If the current filter is valid, return true immediately.
            // Otherwise, check if the provider is enabled when filtering specifically by the given provider name.
            isEnabled: isFilterable || isProviderEnabled
        )
    }

    func getFilteredRelays(_ relayFilter: RelayFilter) -> FilterDescriptor {
        return FilterDescriptor(
            relayFilterResult: RelayCandidates(
                entryRelays: relayCandidatesForAny.entryRelays?.filter {
                    RelaySelector.relayMatchesFilter($0.relay, filter: relayFilter)
                },
                exitRelays: relayCandidatesForAny.exitRelays.filter {
                    RelaySelector.relayMatchesFilter($0.relay, filter: relayFilter)
                }
            ),
            settings: settings
        )
    }

    // MARK: - private Methods

    private func providers(for ownership: RelayFilter.Ownership) -> [String] {
        switch ownership {
        case .any:
            uniqueProviders
        case .owned:
            ownedProviders
        case .rented:
            rentedProviders
        }
    }

    private func toggleProvider(_ name: String) {
        switch relayFilter.providers {
        case .any:
            // If currently "any", switch to only the selected provider
            var providers = providers(for: relayFilter.ownership)
            providers.removeAll { $0 == name }
            relayFilter.providers = .only(providers.map { $0 })
        case var .only(selectedProviders):
            if selectedProviders.contains(name) {
                // If provider exists, remove it
                selectedProviders.removeAll { $0 == name }
            } else {
                // Otherwise, add it
                selectedProviders.append(name)
            }

            // If all available providers are selected, switch back to "any"
            relayFilter.providers = selectedProviders.isEmpty
                ? .only([])
                : (selectedProviders == providers(for: relayFilter.ownership) ? .any : .only(selectedProviders))
        }
    }

    private func extractProviders(from relays: [REST.ServerRelay]) -> [String] {
        Set(relays.map { $0.provider }).caseInsensitiveSorted()
    }

    private func isProviderEnabled(for providerName: String) -> Bool {
        // Check if the provider is enabled when filtering specifically by the given provider name.
        return getFilteredRelays(
            RelayFilter(ownership: relayFilter.ownership, providers: .only([providerName]))
        ).isEnabled
    }
}
