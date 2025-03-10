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
            RelayFilter.Ownership.rented
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
        "Toggles relay filter items correctly",
        arguments: [
            RelayFilterDataSourceItem(name: "DataPacket", type: .provider, isEnabled: true),
            RelayFilterDataSourceItem(name: "All Providers", type: .allProviders, isEnabled: true),
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
    }
}
