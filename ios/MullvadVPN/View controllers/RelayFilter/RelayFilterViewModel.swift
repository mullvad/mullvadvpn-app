//
//  RelayFilterViewModel.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-06-09.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadREST
import MullvadSettings
import MullvadTypes

protocol RelayFilterViewModelSettingsProviding {
    var settings: LatestTunnelSettings { get }
    func addObserver(_ observer: TunnelObserver)
    func removeObserver(_ observer: TunnelObserver)
}
extension TunnelManager: RelayFilterViewModelSettingsProviding {}

extension RelayFilterSelection {
    final class ViewModel: ObservableObject, RelayFilterSettingsViewModelProtocol {
        @Published var relayFilter: RelayFilter
        let multihopContext: MultihopContext
        var onFeatureChipTapped: ((SelectLocationFilter) -> Void)?

        private var settings: LatestTunnelSettings {
            didSet {
                updateFeatureChips()
                objectWillChange.send()
            }
        }
        @Published var chips: [ChipModel] = []
        var filters: [SelectLocationFilter] = []
        private let relaySelectorWrapper: RelaySelectorProtocol
        private let relaysWithLocation: LocationRelays
        private var relayCandidatesForAny: RelayCandidates
        private let tunnelManager: RelayFilterViewModelSettingsProviding
        private var tunnelObserver: TunnelObserver?

        init(
            tunnelManager: RelayFilterViewModelSettingsProviding,
            relaySelectorWrapper: RelaySelectorProtocol,
            multihopContext: MultihopContext
        ) {
            self.tunnelManager = tunnelManager
            self.settings = tunnelManager.settings
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

            let tunnelObserver = TunnelBlockObserver(
                didUpdateTunnelSettings: { [weak self] _, settings in
                    self?.settings = settings
                }
            )
            tunnelManager.addObserver(tunnelObserver)
            self.tunnelObserver = tunnelObserver
            updateFeatureChips()
        }

        deinit {
            guard let tunnelObserver else { return }
            tunnelManager.removeObserver(tunnelObserver)
        }

        private func updateFeatureChips() {
            chips = [
                settings.daita.isEnabled ? .init(id: .daita, name: "Setting: DAITA") : nil,
                settings.wireGuardObfuscation.state.isEnabled
                    ? .init(id: .obfuscation, name: "Setting: \(settings.wireGuardObfuscation.state.description)")
                    : nil,

            ].compactMap { $0 }
            filters = [
                settings.daita.isEnabled ? .daita : nil,
                settings.wireGuardObfuscation.state.isEnabled ? .obfuscation : nil,
            ].compactMap { $0 }

        }

        func onFilterTapped(_ filter: SelectLocationFilter) {
            self.onFeatureChipTapped?(filter)
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

        func toggleItem(_ item: DataSourceItem) {
            switch item.type {
            case .ownershipAny, .ownershipOwned, .ownershipRented:
                relayFilter.ownership = ownership(for: item) ?? .any
            case .allProviders:
                relayFilter.providers = relayFilter.providers == .any ? .only([]) : .any
            case .provider:
                toggleProvider(item.name)
            }
        }

        func availableProviders(for ownership: RelayFilter.Ownership) -> [DataSourceItem] {
            providers(for: ownership)
                .map {
                    providerItem(for: $0)
                }.sorted()
        }

        func ownership(for item: DataSourceItem) -> RelayFilter.Ownership? {
            let ownershipMapping: [DataSourceItem.ItemType: RelayFilter.Ownership] = [
                .ownershipAny: .any,
                .ownershipOwned: .owned,
                .ownershipRented: .rented,
            ]

            return ownershipMapping[item.type]
        }

        func ownershipItem(for ownership: RelayFilter.Ownership) -> DataSourceItem? {
            let ownershipMapping: [RelayFilter.Ownership: DataSourceItem.ItemType] = [
                .any: .ownershipAny,
                .owned: .ownershipOwned,
                .rented: .ownershipRented,
            ]

            return DataSourceItem.ownerships
                .first { $0.type == ownershipMapping[ownership] }
        }

        func providerItem(for providerName: String) -> DataSourceItem {
            let isProviderEnabled = isProviderEnabled(for: providerName)
            let filterDescriptor = getFilteredRelays(relayFilter)

            return DataSourceItem(
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
}

extension RelayFilterSelection.ViewModel {
    struct MockTunnelManager: RelayFilterViewModelSettingsProviding {
        let settings: LatestTunnelSettings
        func addObserver(_ observer: TunnelObserver) {}
        func removeObserver(_ observer: TunnelObserver) {}
    }
}
