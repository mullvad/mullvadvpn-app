//
//  FilterDescriptorTests.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-03-10.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import MullvadMockData
import Testing

@testable import MullvadREST
@testable import MullvadSettings
@testable import MullvadTypes

@Suite("FilterDescriptorTests")
struct FilterDescriptorTests {
    @Test(
        "Returns correct filter descriptor based on settings and relays",
        arguments: [
            (
                LatestTunnelSettings(tunnelMultihopState: .on),
                RelayCandidates(entryRelays: [], exitRelays: createRelayWithLocation()),
                false,
                false,
                MultihopContext.entry
            ),
            (
                LatestTunnelSettings(tunnelMultihopState: .on),
                RelayCandidates(entryRelays: createRelayWithLocation(), exitRelays: createRelayWithLocation()),
                true,
                false,
                MultihopContext.allCases.randomElement().unsafelyUnwrapped
            ),
            (
                LatestTunnelSettings(daita: DAITASettings(daitaState: .on, directOnlyState: .off)),
                RelayCandidates(entryRelays: [], exitRelays: [esMad1]),
                true,
                true,
                MultihopContext.exit
            ),
            (
                LatestTunnelSettings(daita: DAITASettings(daitaState: .on, directOnlyState: .off)),
                RelayCandidates(entryRelays: [esMad1], exitRelays: [seSto6]),
                true,
                true,
                MultihopContext.allCases.randomElement().unsafelyUnwrapped
            ),
            (
                LatestTunnelSettings(daita: DAITASettings(daitaState: .on, directOnlyState: .on)),
                RelayCandidates(entryRelays: nil, exitRelays: [esMad1]),
                true,
                true,
                MultihopContext.exit
            ),
            (
                LatestTunnelSettings(daita: DAITASettings(daitaState: .off, directOnlyState: .off)),
                RelayCandidates(entryRelays: nil, exitRelays: [esMad1, seSto6]),
                true,
                false,
                MultihopContext.exit
            ),
            (
                LatestTunnelSettings(
                    tunnelMultihopState: .on,
                    daita: DAITASettings(daitaState: .off, directOnlyState: .on)
                ),
                RelayCandidates(entryRelays: nil, exitRelays: []),
                false,
                false,
                MultihopContext.allCases.randomElement().unsafelyUnwrapped
            ),
            (
                LatestTunnelSettings(
                    tunnelMultihopState: .on,
                    daita: DAITASettings(daitaState: .off, directOnlyState: .off)
                ),
                RelayCandidates(entryRelays: nil, exitRelays: []),
                false,
                false,
                MultihopContext.allCases.randomElement().unsafelyUnwrapped
            ),
            (
                LatestTunnelSettings(
                    tunnelMultihopState: .on,
                    daita: DAITASettings(daitaState: .on, directOnlyState: .on)
                ),
                RelayCandidates(entryRelays: createRelayWithLocation(), exitRelays: createRelayWithLocation()),
                true,
                true,
                MultihopContext.allCases.randomElement().unsafelyUnwrapped
            ),
            (
                LatestTunnelSettings(
                    tunnelMultihopState: .on,
                    daita: DAITASettings(daitaState: .on, directOnlyState: .off)
                ),
                RelayCandidates(entryRelays: createRelayWithLocation(), exitRelays: createRelayWithLocation()),
                true,
                true,
                MultihopContext.allCases.randomElement().unsafelyUnwrapped
            ),
        ]
    )
    func testFilterDescriptor(
        _ settings: LatestTunnelSettings,
        _ relayCandidates: RelayCandidates,
        _ expectedEnabledState: Bool,
        _ expectedDescription: Bool,
        _ multihopContext: MultihopContext
    ) {
        let filterDescriptor = FilterDescriptor(
            relayFilterResult: relayCandidates,
            settings: settings,
            multihopContext: multihopContext
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

    private static var seSto6: RelayWithLocation<REST.ServerRelay> {
        let sampleRelays = ServerRelaysResponseStubs.sampleRelays
        let relay = sampleRelays.wireguard.relays.first { $0.hostname == "se6-wireguard" }!
        let serverLocation = sampleRelays.locations["se-sto"]!
        let location = Location(
            country: serverLocation.country,
            countryCode: serverLocation.country,
            city: serverLocation.city,
            cityCode: "se-sto",
            latitude: serverLocation.latitude,
            longitude: serverLocation.longitude
        )

        return RelayWithLocation(relay: relay, serverLocation: location)
    }

    private static var esMad1: RelayWithLocation<REST.ServerRelay> {
        let sampleRelays = ServerRelaysResponseStubs.sampleRelays
        let relay = sampleRelays.wireguard.relays.first { $0.hostname == "es1-wireguard" }!
        let serverLocation = sampleRelays.locations["es-mad"]!
        let location = Location(
            country: serverLocation.country,
            countryCode: serverLocation.country,
            city: serverLocation.city,
            cityCode: "es-mad",
            latitude: serverLocation.latitude,
            longitude: serverLocation.longitude
        )

        return RelayWithLocation(relay: relay, serverLocation: location)
    }
}
