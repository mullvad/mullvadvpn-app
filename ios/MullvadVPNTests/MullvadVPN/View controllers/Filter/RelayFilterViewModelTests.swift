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
        let filteredRelays = viewModel.availableRelays

        #expect(filteredRelays.count > 0, "Filtered relays should be enabled")
    }

    @Test(
        "Returns correct providers based on ownership type",
        arguments: [
            RelayFilterItem.anyOwnershipItem(),
            RelayFilterItem.ownedOwnershipItem(),
            RelayFilterItem.rentedOwnershipItem(),
        ]
    )
    func testAvailableProvidersByOwnership(_ ownershipItem: RelayFilterItem) {
        let viewModel = RelayFilterSelection.ViewModel(
            tunnelManager: RelayFilterSelection.ViewModel.MockTunnelManager(settings: LatestTunnelSettings()),
            relaySelectorWrapper: RelaySelectorWrapper(relayCache: MockRelayCache()),
            multihopContext: .exit
        )

        viewModel.toggleItem(ownershipItem)

        #expect(!viewModel.providerItems.isEmpty, "Providers list should not be empty for \(ownershipItem.type)")
    }

    @Test(
        "Toggles relay providers filter items correctly",
        .serialized,
        arguments: [
            RelayFilterItem(name: "DataPacket", type: .provider, isSelected: true),
            RelayFilterItem.allProviders(isSelected: true),
            RelayFilterItem(name: "Blix", type: .provider, isSelected: true),
        ]
    )
    func testToggleFilterItem(_ providerItem: RelayFilterItem) {
        let viewModel = RelayFilterSelection.ViewModel(
            tunnelManager: RelayFilterSelection.ViewModel.MockTunnelManager(settings: LatestTunnelSettings()),
            relaySelectorWrapper: RelaySelectorWrapper(relayCache: MockRelayCache()),
            multihopContext: .exit
        )

        let initialFilter = viewModel.relayFilter
        viewModel.toggleItem(providerItem)
        let updatedFilter = viewModel.relayFilter

        #expect(initialFilter != updatedFilter, "Toggling \(providerItem.name) should change the filter state")

        viewModel.toggleItem(providerItem)

        #expect(viewModel.relayFilter == initialFilter, "Toggling twice should restore the initial state")
    }

    @Test(
        "Toggles relay provider filter items correctly",
        arguments: [
            RelayFilterItem.ownedOwnershipItem(),
            RelayFilterItem.rentedOwnershipItem(),
        ]
    )
    func testToggleRelayProviderFilterItem(_ providerItem: RelayFilterItem) {
        let viewModel = RelayFilterSelection.ViewModel(
            tunnelManager: RelayFilterSelection.ViewModel.MockTunnelManager(settings: LatestTunnelSettings()),
            relaySelectorWrapper: RelaySelectorWrapper(relayCache: MockRelayCache()),
            multihopContext: .exit
        )

        let initialFilter = viewModel.relayFilter
        viewModel.toggleItem(providerItem)
        let updatedFilter = viewModel.relayFilter

        #expect(initialFilter != updatedFilter, "Toggling \(providerItem.name) should update the filter state")
        #expect(
            viewModel.relayFilter.ownership == viewModel.ownership(for: providerItem),
            "Filter's ownership should match the toggled item's ownership"
        )
    }

    @Test(
        "Maps ownership item to the correct ownership filter",
        arguments: [
            RelayFilterItem.anyOwnershipItem(): RelayFilter.Ownership.any,
            RelayFilterItem.ownedOwnershipItem(): RelayFilter.Ownership.owned,
            RelayFilterItem.rentedOwnershipItem(): RelayFilter.Ownership.rented,
        ]
    )
    func testFilterOwnershipForItem(
        _ ownershipItem: RelayFilterItem,
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
