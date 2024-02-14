//
//  TunnelSettingsUpdateTests.swift
//  MullvadVPNTests
//
//  Created by Andrew Bulhak on 2024-02-14.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
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
        let update = TunnelSettingsUpdate.obfuscation(.init(state: .on, port: .port5001))
        update.apply(to: &settings)

        // Then:
        XCTAssertEqual(settings.wireGuardObfuscation, WireGuardObfuscationSettings(state: .on, port: .port5001))
    }

    func testApplyRelayConstraints() {
        // Given:
        var settings = LatestTunnelSettings()

        // When:
        let relayConstraints = RelayConstraints(
            location: .only(.country("zz")),
            port: .only(9999),
            filter: .only(.init(ownership: .rented, providers: .only(["foo", "bar"])))
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
}
