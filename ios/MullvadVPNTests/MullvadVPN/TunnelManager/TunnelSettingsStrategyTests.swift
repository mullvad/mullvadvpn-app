//
//  TunnelSettingsStrategyTests.swift
//  MullvadVPNTests
//
//  Created by Mojgan on 2024-06-19.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//
import MullvadSettings
import MullvadTypes
import XCTest

final class TunnelSettingsStrategyTests: XCTestCase {
    func connectToNewRelayOnMultihopChangesTest() {
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

    func connectToNewRelayOnRelaysConstraintChangeTest() {
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

    func connectToCurrentRelayOnDNSSettingsChangeTest() {
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

    func connectToCurrentRelayOnQuantumResistanceChangesTest() {
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

    func connectToCurrentRelayOnWireGuardObfuscationChangeTest() {
        var currentSettings = LatestTunnelSettings()
        TunnelSettingsUpdate.obfuscation(WireGuardObfuscationSettings(state: .off, port: .port80))
            .apply(to: &currentSettings)

        var updatedSettings = currentSettings
        TunnelSettingsUpdate.obfuscation(WireGuardObfuscationSettings(state: .automatic, port: .automatic))
            .apply(to: &updatedSettings)

        let tunnelSettingsStrategy = TunnelSettingsStrategy()
        XCTAssertTrue(tunnelSettingsStrategy.shouldReconnectToNewRelay(
            oldSettings: currentSettings,
            newSettings: updatedSettings
        ))
    }

    func connectToCurrentRelayWhenNothingChangeTest() {
        let currentSettings = LatestTunnelSettings()
        let updatedSettings = currentSettings

        let tunnelSettingsStrategy = TunnelSettingsStrategy()
        XCTAssertFalse(tunnelSettingsStrategy.shouldReconnectToNewRelay(
            oldSettings: currentSettings,
            newSettings: updatedSettings
        ))
    }
}
