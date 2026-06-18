//
//  RelayFilterViewModelTests.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-03-10.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import MullvadMockData
import Testing

@testable import MullvadREST
@testable import MullvadSettings
@testable import MullvadTypes

struct RelayFilterViewModelTests {
    @Test(
        "Filters relays based on settings",
        arguments: [
            LatestTunnelSettings(),
            LatestTunnelSettings(
                tunnelMultihopState: .never,
                daita: DAITASettings(daitaState: .on)
            ),
            LatestTunnelSettings(
                tunnelMultihopState: .whenNeeded,
                daita: DAITASettings(daitaState: .on)
            ),
            LatestTunnelSettings(
                tunnelMultihopState: .always,
                daita: DAITASettings(daitaState: .on)
            ),
        ]
    )
    func testRelayFiltering(_ settings: LatestTunnelSettings) {
        let viewModel = RelayFilterSelection.ViewModel(
            tunnelManager: RelayFilterSelection.ViewModel.MockTunnelManager(settings: settings),
            relaySelectorWrapper: RelaySelectorWrapper(relayCache: MockRelayCache()),
            multihopContext: .exit
        )
        let filteredRelays = viewModel.getFilteredRelays(RelayFilter())

        #expect(filteredRelays.isEnabled, "Filtered relays should be enabled")
    }

    @Test(
        "Returns correct providers based on ownership type",
        arguments: [
            RelayFilter.Ownership.any,
            RelayFilter.Ownership.owned,
            RelayFilter.Ownership.rented,
        ]
    )
    func testAvailableProvidersByOwnership(_ ownership: RelayFilter.Ownership) {
        let viewModel = RelayFilterSelection.ViewModel(
            tunnelManager: RelayFilterSelection.ViewModel.MockTunnelManager(settings: LatestTunnelSettings()),
            relaySelectorWrapper: RelaySelectorWrapper(relayCache: MockRelayCache()),
            multihopContext: .exit
        )
        let providers = viewModel.availableProviders(for: ownership)

        #expect(!providers.isEmpty, "Providers list should not be empty for \(ownership)")
    }

    @Test(
        "Toggles relay providers filter items correctly",
        arguments: [
            RelayFilterSelection.DataSourceItem(name: "DataPacket", type: .provider, isEnabled: true),
            RelayFilterSelection.DataSourceItem(name: "All Providers", type: .allProviders, isEnabled: true),
            RelayFilterSelection.DataSourceItem(name: "Blix", type: .provider, isEnabled: true),
        ]
    )
    func testToggleFilterItem(_ item: RelayFilterSelection.DataSourceItem) {
        let viewModel = RelayFilterSelection.ViewModel(
            tunnelManager: RelayFilterSelection.ViewModel.MockTunnelManager(settings: LatestTunnelSettings()),
            relaySelectorWrapper: RelaySelectorWrapper(relayCache: MockRelayCache()),
            multihopContext: .exit
        )

        let initialFilter = viewModel.relayFilter
        viewModel.toggleItem(item)
        let updatedFilter = viewModel.relayFilter

        #expect(initialFilter != updatedFilter, "Toggling \(item.name) should change the filter state")

        viewModel.toggleItem(item)

        #expect(viewModel.relayFilter == initialFilter, "Toggling twice should restore the initial state")
    }

    @Test(
        "Toggles relay provider filter items correctly",
        arguments: [
            RelayFilterSelection.DataSourceItem.ownedOwnershipItem,
            RelayFilterSelection.DataSourceItem.rentedOwnershipItem,
        ]
    )
    func testToggleRelayProviderFilterItem(_ item: RelayFilterSelection.DataSourceItem) {
        let viewModel = RelayFilterSelection.ViewModel(
            tunnelManager: RelayFilterSelection.ViewModel.MockTunnelManager(settings: LatestTunnelSettings()),
            relaySelectorWrapper: RelaySelectorWrapper(relayCache: MockRelayCache()),
            multihopContext: .exit
        )

        let initialFilter = viewModel.relayFilter
        viewModel.toggleItem(item)
        let updatedFilter = viewModel.relayFilter

        #expect(initialFilter != updatedFilter, "Toggling \(item.name) should update the filter state")
        #expect(
            viewModel.relayFilter.ownership == viewModel.ownership(for: item),
            "Filter's ownership should match the toggled item's ownership"
        )
    }

    @Test(
        "Maps ownership filter to the correct ownership item",
        arguments: [
            (RelayFilter.Ownership.any, RelayFilterSelection.DataSourceItem.anyOwnershipItem),
            (RelayFilter.Ownership.owned, RelayFilterSelection.DataSourceItem.ownedOwnershipItem),
            (RelayFilter.Ownership.rented, RelayFilterSelection.DataSourceItem.rentedOwnershipItem),
        ]
    )
    func testOwnershipItemForFilter(
        _ ownership: RelayFilter.Ownership,
        expectedItem: RelayFilterSelection.DataSourceItem
    ) throws {
        let viewModel = RelayFilterSelection.ViewModel(
            tunnelManager: RelayFilterSelection.ViewModel.MockTunnelManager(settings: LatestTunnelSettings()),
            relaySelectorWrapper: RelaySelectorWrapper(relayCache: MockRelayCache()),
            multihopContext: .exit
        )
        guard let ownershipItem = viewModel.ownershipItem(for: ownership) else {
            throw TestError.nilOwnershipItem("ownershipItem(for: \(ownership)) returned nil")
        }

        #expect(
            ownershipItem == expectedItem,
            "Expected \(expectedItem.name) for ownership type \(ownership), but got \(ownershipItem.name)"
        )
    }

    @Test(
        "Maps ownership item to the correct ownership filter",
        arguments: [
            (RelayFilterSelection.DataSourceItem.anyOwnershipItem, RelayFilter.Ownership.any),
            (RelayFilterSelection.DataSourceItem.ownedOwnershipItem, RelayFilter.Ownership.owned),
            (RelayFilterSelection.DataSourceItem.rentedOwnershipItem, RelayFilter.Ownership.rented),
        ]
    )
    func testFilterOwnershipForItem(
        _ ownershipItem: RelayFilterSelection.DataSourceItem,
        expectedOwnership: RelayFilter.Ownership
    ) {
        let viewModel = RelayFilterSelection.ViewModel(
            tunnelManager: RelayFilterSelection.ViewModel.MockTunnelManager(settings: LatestTunnelSettings()),
            relaySelectorWrapper: RelaySelectorWrapper(relayCache: MockRelayCache()),
            multihopContext: .exit
        )
        let ownership = viewModel.ownership(for: ownershipItem)

        #expect(
            ownership == expectedOwnership,
            "Expected ownership \(expectedOwnership) for item \(ownershipItem.name), but got \(ownership)"
        )
    }
}

private enum TestError: Error {
    case nilOwnershipItem(String)
}
