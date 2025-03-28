//
//  RelayFilterViewModelTests.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-03-10.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import MullvadMockData
@testable import MullvadREST
@testable import MullvadSettings
@testable import MullvadTypes
import Testing

struct RelayFilterViewModelTests {
    @Test(
        "Filters relays based on settings",
        arguments: [
            LatestTunnelSettings(),
            LatestTunnelSettings(daita: DAITASettings(daitaState: .on, directOnlyState: .on)),
            LatestTunnelSettings(daita: DAITASettings(daitaState: .on, directOnlyState: .off)),
            LatestTunnelSettings(
                tunnelMultihopState: .on,
                daita: DAITASettings(daitaState: .on, directOnlyState: .on)
            ),
            LatestTunnelSettings(
                tunnelMultihopState: .on,
                daita: DAITASettings(daitaState: .on, directOnlyState: .off)
            ),
        ]
    )
    func testRelayFiltering(_ settings: LatestTunnelSettings) {
        let viewModel = RelayFilterViewModel(
            settings: settings,
            relaySelectorWrapper: RelaySelectorWrapper(relayCache: MockRelayCache())
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
        let viewModel = RelayFilterViewModel(
            settings: LatestTunnelSettings(),
            relaySelectorWrapper: RelaySelectorWrapper(relayCache: MockRelayCache())
        )
        let providers = viewModel.availableProviders(for: ownership)

        #expect(!providers.isEmpty, "Providers list should not be empty for \(ownership)")
    }

    @Test(
        "Toggles relay providers filter items correctly",
        arguments: [
            RelayFilterDataSourceItem(name: "DataPacket", type: .provider, isEnabled: true),
            RelayFilterDataSourceItem(name: "All Providers", type: .allProviders, isEnabled: true),
            RelayFilterDataSourceItem(name: "Blix", type: .provider, isEnabled: true),
        ]
    )
    func testToggleFilterItem(_ item: RelayFilterDataSourceItem) {
        let viewModel = RelayFilterViewModel(
            settings: LatestTunnelSettings(),
            relaySelectorWrapper: RelaySelectorWrapper(relayCache: MockRelayCache())
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
            RelayFilterDataSourceItem.ownedOwnershipItem,
            RelayFilterDataSourceItem.rentedOwnershipItem,
        ]
    )
    func testToggleRelayProviderFilterItem(_ item: RelayFilterDataSourceItem) {
        let viewModel = RelayFilterViewModel(
            settings: LatestTunnelSettings(),
            relaySelectorWrapper: RelaySelectorWrapper(relayCache: MockRelayCache())
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
            (RelayFilter.Ownership.any, RelayFilterDataSourceItem.anyOwnershipItem),
            (RelayFilter.Ownership.owned, RelayFilterDataSourceItem.ownedOwnershipItem),
            (RelayFilter.Ownership.rented, RelayFilterDataSourceItem.rentedOwnershipItem),
        ]
    )
    func testOwnershipItemForFilter(
        _ ownership: RelayFilter.Ownership,
        expectedItem: RelayFilterDataSourceItem
    ) throws {
        let viewModel = RelayFilterViewModel(
            settings: LatestTunnelSettings(),
            relaySelectorWrapper: RelaySelectorWrapper(relayCache: MockRelayCache())
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
            (RelayFilterDataSourceItem.anyOwnershipItem, RelayFilter.Ownership.any),
            (RelayFilterDataSourceItem.ownedOwnershipItem, RelayFilter.Ownership.owned),
            (RelayFilterDataSourceItem.rentedOwnershipItem, RelayFilter.Ownership.rented),
        ]
    )
    func testFilterOwnershipForItem(
        _ ownershipItem: RelayFilterDataSourceItem,
        expectedOwnership: RelayFilter.Ownership
    ) {
        let viewModel = RelayFilterViewModel(
            settings: LatestTunnelSettings(),
            relaySelectorWrapper: RelaySelectorWrapper(relayCache: MockRelayCache())
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
