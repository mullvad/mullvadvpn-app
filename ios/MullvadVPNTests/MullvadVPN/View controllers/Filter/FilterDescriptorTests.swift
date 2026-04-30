//
//  FilterDescriptorTests.swift
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

@Suite("FilterDescriptorTests")
struct FilterDescriptorTests {
    enum FilterDescription: String {
        case daita = "When using DAITA, one provider with DAITA-enabled servers is required"
        case disabled = "Filters are overridden when using the automatic location"
        case none = ""

        var shortDescription: String {
            switch self {
            case .daita:
                ".daita"
            case .disabled:
                ".disabled"
            case .none:
                ".none"
            }
        }
    }

    @Test(
        "Returns correct filter descriptor based on settings and relays",
        arguments: [
            (
                LatestTunnelSettings(tunnelMultihopState: .always),
                RelayCandidates(entryRelays: [], exitRelays: createRelayWithLocation()),
                false,
                [FilterDescription.none],
                MultihopContext.entry
            ),
            (
                LatestTunnelSettings(tunnelMultihopState: .always),
                RelayCandidates(entryRelays: createRelayWithLocation(), exitRelays: createRelayWithLocation()),
                true,
                [FilterDescription.none],
                MultihopContext.allCases.randomElement().unsafelyUnwrapped
            ),
            // DAITA on + automatic routing: exit doesn't need any description
            (
                LatestTunnelSettings(
                    tunnelMultihopState: .whenNeeded,
                    daita: DAITASettings(daitaState: .on)
                ),
                RelayCandidates(entryRelays: [], exitRelays: [esMad1]),
                true,
                [FilterDescription.none],
                MultihopContext.exit
            ),
            // DAITA on + automatic routing: entry needs "disabled" description
            (
                LatestTunnelSettings(
                    tunnelMultihopState: .whenNeeded,
                    daita: DAITASettings(daitaState: .on)
                ),
                RelayCandidates(entryRelays: [esMad1], exitRelays: [seSto6]),
                true,
                [FilterDescription.disabled, FilterDescription.daita],
                MultihopContext.entry
            ),
            // DAITA on + automatic routing: exit doesn't need any description
            (
                LatestTunnelSettings(
                    tunnelMultihopState: .whenNeeded,
                    daita: DAITASettings(daitaState: .on)
                ),
                RelayCandidates(entryRelays: [esMad1], exitRelays: [seSto6]),
                true,
                [FilterDescription.none],
                MultihopContext.exit
            ),
            // DAITA on: exit needs DAITA description (no auto routing, no multihop)
            (
                LatestTunnelSettings(daita: DAITASettings(daitaState: .on)),
                RelayCandidates(entryRelays: nil, exitRelays: [esMad1]),
                true,
                [FilterDescription.daita],
                MultihopContext.exit
            ),
            (
                LatestTunnelSettings(daita: DAITASettings(daitaState: .off)),
                RelayCandidates(entryRelays: nil, exitRelays: [esMad1, seSto6]),
                true,
                [FilterDescription.none],
                MultihopContext.exit
            ),
            (
                LatestTunnelSettings(
                    tunnelMultihopState: .always,
                    daita: DAITASettings(daitaState: .off)
                ),
                RelayCandidates(entryRelays: nil, exitRelays: []),
                false,
                [FilterDescription.none],
                MultihopContext.allCases.randomElement().unsafelyUnwrapped
            ),
            // Multihop + DAITA: entry shows DAITA description
            (
                LatestTunnelSettings(
                    tunnelMultihopState: .always,
                    daita: DAITASettings(daitaState: .on)
                ),
                RelayCandidates(entryRelays: createRelayWithLocation(), exitRelays: createRelayWithLocation()),
                true,
                [FilterDescription.daita],
                MultihopContext.entry
            ),
            // Multihop + DAITA: exit does not show description
            (
                LatestTunnelSettings(
                    tunnelMultihopState: .always,
                    daita: DAITASettings(daitaState: .on)
                ),
                RelayCandidates(entryRelays: createRelayWithLocation(), exitRelays: createRelayWithLocation()),
                true,
                [FilterDescription.none],
                MultihopContext.exit
            ),
        ]
    )
    func testFilterDescriptor(
        _ settings: LatestTunnelSettings,
        _ relayCandidates: RelayCandidates,
        _ expectedEnabledState: Bool,
        _ expectedDescriptions: [FilterDescription],
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
            filterDescriptor.descriptions == expectedDescriptions.compactMap { $0 == .none ? nil : $0.rawValue },
            "Description should be \(expectedDescriptions.map { $0.shortDescription })"
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
