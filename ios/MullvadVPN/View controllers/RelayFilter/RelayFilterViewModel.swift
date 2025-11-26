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
    let multihopContext: MultihopContext

    private var settings: LatestTunnelSettings
    private let relaySelectorWrapper: RelaySelectorProtocol
    private let relaysWithLocation: LocationRelays
    private var relayCandidatesForAny: RelayCandidates

    init(
        settings: LatestTunnelSettings,
        relaySelectorWrapper: RelaySelectorProtocol,
        multihopContext: MultihopContext
    ) {
        self.settings = settings
        var settingsCopy = settings

        self.relaySelectorWrapper = relaySelectorWrapper
        self.multihopContext = multihopContext

        switch multihopContext {
        case .entry:
            relayFilter = settings.relayConstraints.entryFilter.value ?? RelayFilter()
            settingsCopy.relayConstraints.entryFilter = .any
        case .exit:
            relayFilter = settings.relayConstraints.exitFilter.value ?? RelayFilter()
            settingsCopy.relayConstraints.exitFilter = .any
        }

        // Retrieve all available relays that satisfy the `any` constraint.
        // This constraint ensures that the selected relays are associated with the current tunnel settings
        // and serve as the primary source of truth for subsequent filtering operations.
        // Further filtering will be applied based on specific criteria such as `ownership` or `provider`.
        if let relayCandidatesForAny = try? relaySelectorWrapper.findCandidates(tunnelSettings: settingsCopy) {
            self.relayCandidatesForAny = relayCandidatesForAny
        } else {
            relayCandidatesForAny = RelayCandidates(entryRelays: nil, exitRelays: [])
        }

        // Directly setting relaysWithLocation in constructor
        if let cachedResponse = try? relaySelectorWrapper.relayCache.read().relays {
            relaysWithLocation = LocationRelays(
                relays: cachedResponse.wireguard.relays,
                locations: cachedResponse.locations
            )
        } else {
            relaysWithLocation = LocationRelays(relays: [], locations: [:])
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

    func toggleItem(_ item: RelayFilterDataSourceItem) {
        switch item.type {
        case .ownershipAny, .ownershipOwned, .ownershipRented:
            relayFilter.ownership = ownership(for: item) ?? .any
        case .allProviders:
            relayFilter.providers = relayFilter.providers == .any ? .only([]) : .any
        case .provider:
            toggleProvider(item.name)
        }
    }

    func availableProviders(for ownership: RelayFilter.Ownership) -> [RelayFilterDataSourceItem] {
        providers(for: ownership)
            .map {
                providerItem(for: $0)
            }.sorted()
    }

    func ownership(for item: RelayFilterDataSourceItem) -> RelayFilter.Ownership? {
        let ownershipMapping: [RelayFilterDataSourceItem.ItemType: RelayFilter.Ownership] = [
            .ownershipAny: .any,
            .ownershipOwned: .owned,
            .ownershipRented: .rented,
        ]

        return ownershipMapping[item.type]
    }

    func ownershipItem(for ownership: RelayFilter.Ownership) -> RelayFilterDataSourceItem? {
        let ownershipMapping: [RelayFilter.Ownership: RelayFilterDataSourceItem.ItemType] = [
            .any: .ownershipAny,
            .owned: .ownershipOwned,
            .rented: .ownershipRented,
        ]

        return RelayFilterDataSourceItem.ownerships
            .first { $0.type == ownershipMapping[ownership] }
    }

    func providerItem(for providerName: String) -> RelayFilterDataSourceItem {
        let isProviderEnabled = isProviderEnabled(for: providerName)
        let filterDescriptor = getFilteredRelays(relayFilter)

        return RelayFilterDataSourceItem(
            name: providerName,
            description: filterDescriptor.shouldShowDaitaDescription && isProviderEnabled
                ? String(format: NSLocalizedString("%@-enabled", comment: ""), "DAITA")
                : "",
            type: .provider,
            isEnabled: filterDescriptor.isEnabled && isProviderEnabled
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
            settings: settings,
            multihopContext: multihopContext
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
            relayFilter.providers =
                selectedProviders.isEmpty
                ? .only([])
                : (selectedProviders.count == providers(for: relayFilter.ownership).count
                    ? .any
                    : .only(selectedProviders))
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
