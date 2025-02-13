//
//  TunnelSettingsStrategyTests.swift
//  MullvadVPNTests
//
//  Created by Mojgan on 2024-06-19.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import MullvadSettings
import MullvadTypes
import XCTest

final class TunnelSettingsStrategyTests: XCTestCase {
    func testConnectToNewRelayOnMultihopChanges() {
        var currentSettings = LatestTunnelSettings()
        TunnelSettingsUpdate.multihop(.off).apply(to: &currentSettings)

        var updatedSettings = currentSettings
        TunnelSettingsUpdate.multihop(.on).apply(to: &updatedSettings)

        let tunnelSettingsStrategy = TunnelSettingsStrategy()
        XCTAssertTrue(tunnelSettingsStrategy.shouldReconnectToNewRelay(
            oldSettings: currentSettings,
            newSettings: updatedSettings
        ))
    }

    func testConnectToNewRelayOnRelaysConstraintChange() {
        var currentSettings = LatestTunnelSettings()
        TunnelSettingsUpdate.relayConstraints(RelayConstraints()).apply(to: &currentSettings)

        var updatedSettings = currentSettings
        TunnelSettingsUpdate.relayConstraints(RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.country("zz")])),
            port: .only(9999),
            filter: .only(.init(ownership: .rented, providers: .only(["foo", "bar"])))
        )).apply(to: &updatedSettings)

        let tunnelSettingsStrategy = TunnelSettingsStrategy()
        XCTAssertTrue(tunnelSettingsStrategy.shouldReconnectToNewRelay(
            oldSettings: currentSettings,
            newSettings: updatedSettings
        ))
    }

    func testConnectToCurrentRelayOnDNSSettingsChange() {
        let currentSettings = LatestTunnelSettings()

        var updatedSettings = currentSettings
        var dnsSettings = DNSSettings()
        dnsSettings.blockingOptions = [.blockAdvertising, .blockTracking]
        dnsSettings.enableCustomDNS = true
        TunnelSettingsUpdate.dnsSettings(dnsSettings).apply(to: &updatedSettings)

        let tunnelSettingsStrategy = TunnelSettingsStrategy()
        XCTAssertTrue(tunnelSettingsStrategy.shouldReconnectToNewRelay(
            oldSettings: currentSettings,
            newSettings: updatedSettings
        ))
    }

    func testConnectToCurrentRelayOnQuantumResistanceChanges() {
        var currentSettings = LatestTunnelSettings()
        TunnelSettingsUpdate.quantumResistance(.off).apply(to: &currentSettings)

        var updatedSettings = currentSettings
        TunnelSettingsUpdate.quantumResistance(.on).apply(to: &updatedSettings)

        let tunnelSettingsStrategy = TunnelSettingsStrategy()
        XCTAssertTrue(tunnelSettingsStrategy.shouldReconnectToNewRelay(
            oldSettings: currentSettings,
            newSettings: updatedSettings
        ))
    }

    func testConnectToCurrentRelayOnWireGuardObfuscationChange() {
        var currentSettings = LatestTunnelSettings()
        TunnelSettingsUpdate.obfuscation(WireGuardObfuscationSettings(
            state: .off,
            udpOverTcpPort: .port80
        ))
        .apply(to: &currentSettings)

        var updatedSettings = currentSettings
        TunnelSettingsUpdate.obfuscation(WireGuardObfuscationSettings(
            state: .automatic,
            udpOverTcpPort: .automatic
        ))
        .apply(to: &updatedSettings)

        let tunnelSettingsStrategy = TunnelSettingsStrategy()
        XCTAssertTrue(tunnelSettingsStrategy.shouldReconnectToNewRelay(
            oldSettings: currentSettings,
            newSettings: updatedSettings
        ))
    }

    func testConnectToCurrentRelayWhenNothingChange() {
        let currentSettings = LatestTunnelSettings()
        let updatedSettings = currentSettings

        let tunnelSettingsStrategy = TunnelSettingsStrategy()
        XCTAssertFalse(tunnelSettingsStrategy.shouldReconnectToNewRelay(
            oldSettings: currentSettings,
            newSettings: updatedSettings
        ))
    }

    func testHardReconnectWhenIncludeAllNetworksChange() {
        let currentSettings = LatestTunnelSettings()
        var updatedSettings = currentSettings
        TunnelSettingsUpdate.includeAllNetworks(true)
            .apply(to: &updatedSettings)

        let tunnelSettingsStrategy = TunnelSettingsStrategy()
        XCTAssertEqual(tunnelSettingsStrategy.getReconnectionStrategy(
            oldSettings: currentSettings,
            newSettings: updatedSettings
        ), .hardReconnect)
    }

    func testHardReconnectWhenLocalNetworkSharingChange() {
        let currentSettings = LatestTunnelSettings()
        var updatedSettings = currentSettings
        TunnelSettingsUpdate.localNetworkSharing(true)
            .apply(to: &updatedSettings)

        let tunnelSettingsStrategy = TunnelSettingsStrategy()
        XCTAssertEqual(tunnelSettingsStrategy.getReconnectionStrategy(
            oldSettings: currentSettings,
            newSettings: updatedSettings
        ), .hardReconnect)
    }
}
