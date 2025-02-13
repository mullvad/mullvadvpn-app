//
//  TunnelSettingsUpdateTests.swift
//  MullvadVPNTests
//
//  Created by Andrew Bulhak on 2024-02-14.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadSettings
import MullvadTypes
import Network
import XCTest

final class TunnelSettingsUpdateTests: XCTestCase {
    func testApplyDNSSettings() {
        // Given:
        var settings = LatestTunnelSettings()

        // When:
        var dnsSettings = DNSSettings()
        dnsSettings.blockingOptions = [.blockAdvertising, .blockTracking]
        dnsSettings.enableCustomDNS = true
        dnsSettings.customDNSDomains = [.ipv4(IPv4Address("1.2.3.4")!)]
        let update = TunnelSettingsUpdate.dnsSettings(dnsSettings)
        update.apply(to: &settings)

        // Then:
        XCTAssertEqual(settings.dnsSettings.blockingOptions, [.blockAdvertising, .blockTracking])
        XCTAssertEqual(settings.dnsSettings.enableCustomDNS, true)
        XCTAssertEqual(settings.dnsSettings.customDNSDomains, [.ipv4(IPv4Address("1.2.3.4")!)])
    }

    func testApplyObfuscation() {
        // Given:
        var settings = LatestTunnelSettings()

        // When:
        let update = TunnelSettingsUpdate.obfuscation(WireGuardObfuscationSettings(
            state: .udpOverTcp,
            udpOverTcpPort: .port5001
        ))
        update.apply(to: &settings)

        // Then:
        XCTAssertEqual(settings.wireGuardObfuscation, WireGuardObfuscationSettings(
            state: .udpOverTcp,
            udpOverTcpPort: .port5001
        ))
    }

    func testApplyRelayConstraints() {
        // Given:
        var settings = LatestTunnelSettings()

        // When:
        let relayConstraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.country("zz")])),
            port: .only(9999),
            filter: .only(RelayFilter(ownership: .rented, providers: .only(["foo", "bar"])))
        )
        let update = TunnelSettingsUpdate.relayConstraints(relayConstraints)
        update.apply(to: &settings)

        // Then:
        XCTAssertEqual(settings.relayConstraints, relayConstraints)
    }

    func testApplyQuantumResistance() {
        // Given:
        var settings = LatestTunnelSettings()

        // When:
        let update = TunnelSettingsUpdate.quantumResistance(.on)
        update.apply(to: &settings)

        // Then:
        XCTAssertEqual(settings.tunnelQuantumResistance, .on)
    }

    func testApplyMultihop() {
        // Given:
        var settings = LatestTunnelSettings()

        // When:
        let update = TunnelSettingsUpdate.multihop(.on)
        update.apply(to: &settings)

        // Then:
        XCTAssertEqual(settings.tunnelMultihopState, .on)
    }

    func testApplyDAITA() {
        // Given:
        let daitaSettings = DAITASettings(daitaState: .on)
        var settings = LatestTunnelSettings()

        // When:
        let update = TunnelSettingsUpdate.daita(daitaSettings)
        update.apply(to: &settings)

        // Then:
        XCTAssertEqual(settings.daita, daitaSettings)
    }

    func testApplyIAN() {
        // Given:
        let includeAllNetworks = true
        var settings = LatestTunnelSettings()

        // When:
        let update = TunnelSettingsUpdate.includeAllNetworks(includeAllNetworks)
        update.apply(to: &settings)

        // Then:
        XCTAssertEqual(settings.includeAllNetworks, includeAllNetworks)
    }

    func testApplyLocalNetworkSharing() {
        // Given:
        let localNetworkSharing = true
        var settings = LatestTunnelSettings()

        // When:
        let update = TunnelSettingsUpdate.localNetworkSharing(
            localNetworkSharing
        )
        update.apply(to: &settings)

        // Then:
        XCTAssertEqual(settings.localNetworkSharing, localNetworkSharing)
    }
}
