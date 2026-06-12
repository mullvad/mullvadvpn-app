//
//  RelayFilterViewModel.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-06-09.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadSettings
import MullvadTypes

protocol RelayFilterSettingsViewModelProtocol {
    var featureFilters: [SelectLocationFilter] { get }
    var shouldShowAutomaticFilterOverrideNotice: Bool { get }
    var onFeatureChipTapped: ((SelectLocationFilter) -> Void)? { get }
}

protocol RelayFilterViewModelSettingsProviding {
    var settings: LatestTunnelSettings { get }
    func addObserver(_ observer: TunnelObserver)
    func removeObserver(_ observer: TunnelObserver)
}
extension TunnelManager: RelayFilterViewModelSettingsProviding {}

extension RelayFilterSelection {
    final class ViewModel: ObservableObject, RelayFilterSettingsViewModelProtocol {
        // View state holders.
        let multihopContext: MultihopContext
        var ownershipItems: [RelayFilterItem] = []
        var providerItems: [RelayFilterItem] {
            get { filteredProviderItems[selectedOwnership] ?? [] }
            set { filteredProviderItems[selectedOwnership] = newValue }
        }
        var availableRelays: [RelayWithLocation<REST.ServerRelay>] {
            relays.filter { relay in
                providerItems.contains { providerItem in
                    (providerItem.name == relay.relay.provider) && providerItem.isSelected
                }
            }
        }
        var shouldShowAutomaticFilterOverrideNotice: Bool = false

        // Callbacks
        var onApplyFilter: ((RelayFilter) -> Void)?
        var onFeatureChipTapped: ((SelectLocationFilter) -> Void)?
        var onCancel: (() -> Void)?

        // Dependencies
        private let relaySelectorWrapper: RelaySelectorProtocol
        private let tunnelManager: RelayFilterViewModelSettingsProviding
        private var tunnelObserver: TunnelObserver?
        private var settings: LatestTunnelSettings {
            tunnelManager.settings
        }

        // Internal state holders.
        var featureFilters: [SelectLocationFilter] = []
        var relayFilter: RelayFilter
        private var relays: [RelayWithLocation<REST.ServerRelay>] = []
        private var filteredProviderItems: [RelayFilter.Ownership: [RelayFilterItem]] = [:]
        private var selectedOwnership: RelayFilter.Ownership {
            ownership(for: ownershipItems.first { $0.isSelected })
        }

        init(
            tunnelManager: RelayFilterViewModelSettingsProviding,
            relaySelectorWrapper: RelaySelectorProtocol,
            multihopContext: MultihopContext
        ) {
            self.relaySelectorWrapper = relaySelectorWrapper
            self.multihopContext = multihopContext
            self.tunnelManager = tunnelManager

            // Load filter settings from store.
            relayFilter =
                switch multihopContext {
                case .entry:
                    tunnelManager.settings.relayConstraints.entryFilter.value ?? RelayFilter()
                case .exit:
                    tunnelManager.settings.relayConstraints.exitFilter.value ?? RelayFilter()
                }

            // Set up listener to update view when settings change, eg. DAITA is toggled.
            let tunnelObserver = TunnelBlockObserver(
                didUpdateTunnelSettings: { [weak self] _, _ in
                    guard let self else { return }
                    objectWillChange.send()
                    reloadAllData()
                }
            )
            tunnelManager.addObserver(tunnelObserver)
            self.tunnelObserver = tunnelObserver

            // Automatic override notice should be visible when showing entry filter and an automatic location is selected.
            shouldShowAutomaticFilterOverrideNotice = multihopContext == .entry && settings.automaticMultihopIsEnabled

            reloadAllData()
        }

        deinit {
            guard let tunnelObserver else { return }
            tunnelManager.removeObserver(tunnelObserver)
        }

        // MARK: - Public methods

        func toggleItem(_ item: RelayFilterItem) {
            objectWillChange.send()

            if [.ownershipAny, .ownershipOwned, .ownershipRented].contains(item.type) {
                // Update ownership items.
                ownershipItems.forEach { $0.isSelected = $0 == item }
            } else if let item = providerItems.first(where: { $0 == item }) {
                // Update toggled provider item.
                item.isSelected.toggle()

                // Update all items if `.allProviders` is toggled.
                if item.type == .allProviders {
                    providerItems.forEach { $0.isSelected = item.isSelected }
                }
            }

            // Get the `.allProviders` item and determine if it should be selected.
            let allProvidersItem = providerItems.first
            allProvidersItem?.isSelected = providerItems.dropFirst().allSatisfy(\.isSelected)

            applyFilter()
        }

        func applyFilter() {
            // Get all provider items, removing the `.allProviders` item.
            let allProviders = providerItems.dropFirst()
            let selectedProviders = allProviders.filter { $0.isSelected }

            relayFilter = RelayFilter(
                ownership: selectedOwnership,
                providers: selectedProviders.count == allProviders.count
                    // If all visible provider items are selected, set `.any` since the
                    // intention is to allow all providers.
                    ? .any
                    // Otherwise get selected providers.
                    : .only(selectedProviders.map { $0.name })
            )
        }

        // MARK: - Private methods

        private func reloadAllData() {
            reloadRelays()
            reloadFeatureChips()
            reloadOwnershipItems()
            reloadProviderItems()
        }

        private func reloadRelays() {
            // Reset relay constraints so that we can select from all available relays.
            var settings = tunnelManager.settings
            settings.relayConstraints = .init(
                entryLocations: .any,
                exitLocations: .any
            )

            // Fetch relays based on settings.
            let relayCandidates = try? relaySelectorWrapper.findCandidates(
                tunnelSettings: settings,
                includeInactive: false
            )

            relays =
                switch multihopContext {
                case .entry:
                    relayCandidates?.entryRelays ?? []
                case .exit:
                    relayCandidates?.exitRelays ?? []
                }
        }

        private func reloadOwnershipItems() {
            ownershipItems = [
                .anyOwnershipItem(isSelected: relayFilter.ownership == .any),
                .ownedOwnershipItem(isSelected: relayFilter.ownership == .owned),
                .rentedOwnershipItem(isSelected: relayFilter.ownership == .rented),
            ]
        }

        private func reloadProviderItems() {
            var anyProviders = [RelayFilterItem]()
            var ownedProviders = [RelayFilterItem]()
            var rentedProviders = [RelayFilterItem]()

            // It may seem a little roundabout to first try to fetch an item from `.anyProviders` list and then check
            // again if it exists before adding it. This happens because we want to reuse the same item for all
            // collections so that selection status of an item is shared between them. Also, a provider might be part
            // of both owned and rented, making things even more complicated. The flow below ensures we cover all cases.
            let selectedProviders = relayFilter.providers
            relays.forEach { relay in
                let providerName = relay.relay.provider

                // Fetch existing or create new item.
                let providerItem =
                    anyProviders.first(where: { $0.name == providerName })
                    ?? RelayFilterItem(
                        name: providerName,
                        type: .provider,
                        isSelected: false
                    )

                // Update selection based on settings in store.
                providerItem.isSelected =
                    selectedProviders == .any || (selectedProviders.value ?? []).contains(providerName)

                // Add to `.anyOwnershipItem` list.
                if !anyProviders.contains(providerItem) {
                    anyProviders.append(providerItem)
                }

                if relay.relay.owned {
                    // Add to `.ownedOwnershipItem` list.
                    if !ownedProviders.contains(providerItem) {
                        ownedProviders.append(providerItem)
                    }
                } else {
                    // Add to `.rentedOwnershipItem` list.
                    if !rentedProviders.contains(providerItem) {
                        rentedProviders.append(providerItem)
                    }
                }
            }

            // Add lists to their respective ownership items. Also, prepend the `.allProviders` item to each.
            filteredProviderItems[.any] =
                [.allProviders(isSelected: anyProviders.allSatisfy(\.isSelected))] + anyProviders.sorted()
            filteredProviderItems[.owned] =
                [.allProviders(isSelected: ownedProviders.allSatisfy(\.isSelected))] + ownedProviders.sorted()
            filteredProviderItems[.rented] =
                [.allProviders(isSelected: rentedProviders.allSatisfy(\.isSelected))] + rentedProviders.sorted()
        }

        internal func ownership(for item: RelayFilterItem?) -> RelayFilter.Ownership {
            switch item?.type {
            case .ownershipAny:
                .any
            case .ownershipOwned:
                .owned
            case .ownershipRented:
                .rented
            default:
                .any
            }
        }

        private func reloadFeatureChips() {
            let featurePills: [SelectLocationFilter] =
                [
                    settings.daita.isEnabled ? .daita : nil,
                    settings.wireGuardObfuscation.state.isEnabled
                        ? .obfuscation(settings.wireGuardObfuscation.state)
                        : nil,
                ].compactMap { $0 }

            featureFilters =
                switch (multihopContext, settings.automaticMultihopIsEnabled) {
                case (.entry, true), (.exit, true):
                    []
                case (.entry, false):
                    featurePills
                case (.exit, false):
                    tunnelManager.settings.tunnelMultihopState == .never ? featurePills : []
                }
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
