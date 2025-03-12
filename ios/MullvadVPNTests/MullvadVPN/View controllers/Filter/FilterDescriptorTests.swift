//
//  FilterDescriptorTests.swift
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

@Suite("FilterDescriptorTests")
struct FilterDescriptorTests {
    @Test(
        "Returns correct filter descriptor based on settings and relays",
        arguments: [
            (
                LatestTunnelSettings(tunnelMultihopState: .on),
                RelayCandidates(entryRelays: [], exitRelays: createRelayWithLocation()),
                false,
                false
            ),
            (
                LatestTunnelSettings(tunnelMultihopState: .on),
                RelayCandidates(entryRelays: createRelayWithLocation(), exitRelays: createRelayWithLocation()),
                true,
                false
            ),
            (
                LatestTunnelSettings(daita: DAITASettings(daitaState: .on, directOnlyState: .off)),
                RelayCandidates(entryRelays: [], exitRelays: createRelayWithLocation()),
                false,
                true
            ),
            (
                LatestTunnelSettings(daita: DAITASettings(daitaState: .on, directOnlyState: .off)),
                RelayCandidates(entryRelays: createRelayWithLocation(), exitRelays: createRelayWithLocation()),
                true,
                true
            ),
            (
                LatestTunnelSettings(daita: DAITASettings(daitaState: .on, directOnlyState: .on)),
                RelayCandidates(entryRelays: nil, exitRelays: createRelayWithLocation()),
                true,
                true
            ),
            (
                LatestTunnelSettings(daita: DAITASettings(daitaState: .on, directOnlyState: .off)),
                RelayCandidates(entryRelays: nil, exitRelays: createRelayWithLocation()),
                false,
                true
            ),
            (
                LatestTunnelSettings(daita: DAITASettings(daitaState: .off, directOnlyState: .off)),
                RelayCandidates(entryRelays: nil, exitRelays: []),
                false,
                false
            ),
            (
                LatestTunnelSettings(
                    tunnelMultihopState: .on,
                    daita: DAITASettings(daitaState: .off, directOnlyState: .on)
                ),
                RelayCandidates(entryRelays: nil, exitRelays: []),
                false,
                false
            ),
            (
                LatestTunnelSettings(
                    tunnelMultihopState: .on,
                    daita: DAITASettings(daitaState: .off, directOnlyState: .off)
                ),
                RelayCandidates(entryRelays: nil, exitRelays: []),
                false,
                false
            ),
            (
                LatestTunnelSettings(
                    tunnelMultihopState: .on,
                    daita: DAITASettings(daitaState: .on, directOnlyState: .on)
                ),
                RelayCandidates(entryRelays: createRelayWithLocation(), exitRelays: createRelayWithLocation()),
                true,
                true
            ),
            (
                LatestTunnelSettings(
                    tunnelMultihopState: .on,
                    daita: DAITASettings(daitaState: .on, directOnlyState: .off)
                ),
                RelayCandidates(entryRelays: createRelayWithLocation(), exitRelays: createRelayWithLocation()),
                true,
                true
            ),
        ]
    )
    func testFilterDescriptor(
        _ settings: LatestTunnelSettings,
        _ relayCandidates: RelayCandidates,
        _ expectedEnabledState: Bool,
        _ expectedDescription: Bool
    ) {
        let filterDescriptor = FilterDescriptor(
            relayFilterResult: relayCandidates,
            settings: settings
        )

        #expect(
            filterDescriptor.isEnabled == expectedEnabledState,
            "Expected filter descriptor to be \(expectedEnabledState ? "enabled" : "disabled")"
        )
        #expect(
            (filterDescriptor.title.rangeOfCharacter(from: .decimalDigits) != nil) == expectedEnabledState,
            "Title should contain numbers only when enabled"
        )
        #expect(
            filterDescriptor.description.isEmpty != expectedDescription,
            "Description should \(expectedDescription ? "not be empty" : "be empty")"
        )
    }

    // Helper function to generate relay locations
    private static func createRelayWithLocation() -> [RelayWithLocation<REST.ServerRelay>] {
        let sampleRelays = ServerRelaysResponseStubs.sampleRelays
        return sampleRelays.wireguard.relays.map { relay in
            let location = sampleRelays.locations[relay.location.rawValue]!

            return RelayWithLocation(
                relay: relay,
                serverLocation: Location(
                    country: location.country,
                    countryCode: String(relay.location.country),
                    city: location.city,
                    cityCode: String(relay.location.city),
                    latitude: location.latitude,
                    longitude: location.longitude
                )
            )
        }
    }
}
