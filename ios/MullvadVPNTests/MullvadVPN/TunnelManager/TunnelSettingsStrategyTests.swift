// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only
import MullvadSettings
import MullvadTypes
import XCTest

final class TunnelSettingsStrategyTests: XCTestCase {
    func testConnectToNewRelayOnMultihopChanges() {
        var currentSettings = LatestTunnelSettings()
        TunnelSettingsUpdate.multihop(.never).apply(to: &currentSettings)

        var updatedSettings = currentSettings
        TunnelSettingsUpdate.multihop(.always).apply(to: &updatedSettings)

        let tunnelSettingsStrategy = TunnelSettingsStrategy()
        XCTAssertTrue(
            tunnelSettingsStrategy.shouldReconnectToNewRelay(
                oldSettings: currentSettings,
                newSettings: updatedSettings
            ))
    }

    func testConnectToNewRelayOnRelaysConstraintChange() {
        var currentSettings = LatestTunnelSettings()
        TunnelSettingsUpdate.relayConstraints(RelayConstraints()).apply(to: &currentSettings)

        let filter: RelayConstraint = .only(RelayFilter(ownership: .rented, providers: .only(["foo", "bar"])))
        var updatedSettings = currentSettings
        TunnelSettingsUpdate.relayConstraints(
            RelayConstraints(
                exitLocations: .only(UserSelectedRelays(locations: [.country("zz")])),
                port: .only(9999),
                entryFilter: filter,
                exitFilter: filter
            )
        ).apply(to: &updatedSettings)

        let tunnelSettingsStrategy = TunnelSettingsStrategy()
        XCTAssertTrue(
            tunnelSettingsStrategy.shouldReconnectToNewRelay(
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
        XCTAssertTrue(
            tunnelSettingsStrategy.shouldReconnectToNewRelay(
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
        XCTAssertTrue(
            tunnelSettingsStrategy.shouldReconnectToNewRelay(
                oldSettings: currentSettings,
                newSettings: updatedSettings
            ))
    }

    func testConnectToCurrentRelayOnWireGuardObfuscationChange() {
        var currentSettings = LatestTunnelSettings()
        TunnelSettingsUpdate.obfuscation(
            WireGuardObfuscationSettings(
                state: .off,
                udpOverTcpPort: .port80
            )
        )
        .apply(to: &currentSettings)

        var updatedSettings = currentSettings
        TunnelSettingsUpdate.obfuscation(
            WireGuardObfuscationSettings(
                state: .automatic,
                udpOverTcpPort: .automatic
            )
        )
        .apply(to: &updatedSettings)

        let tunnelSettingsStrategy = TunnelSettingsStrategy()
        XCTAssertTrue(
            tunnelSettingsStrategy.shouldReconnectToNewRelay(
                oldSettings: currentSettings,
                newSettings: updatedSettings
            ))
    }

    func testConnectToCurrentRelayWhenNothingChange() {
        let currentSettings = LatestTunnelSettings()
        let updatedSettings = currentSettings

        let tunnelSettingsStrategy = TunnelSettingsStrategy()
        XCTAssertFalse(
            tunnelSettingsStrategy.shouldReconnectToNewRelay(
                oldSettings: currentSettings,
                newSettings: updatedSettings
            ))
    }

    func testHardReconnectWhenIncludeAllNetworksChange() {
        let currentSettings = LatestTunnelSettings()
        var updatedSettings = currentSettings
        TunnelSettingsUpdate.includeAllNetworks(
            IncludeAllNetworksSettings(
                includeAllNetworksState: .on
            )
        )
        .apply(to: &updatedSettings)

        let tunnelSettingsStrategy = TunnelSettingsStrategy()
        XCTAssertEqual(
            tunnelSettingsStrategy.getReconnectionStrategy(
                oldSettings: currentSettings,
                newSettings: updatedSettings
            ), .hardReconnect)
    }

    func testHardReconnectWhenLocalNetworkSharingChange() {
        let currentSettings = LatestTunnelSettings()
        var updatedSettings = currentSettings
        TunnelSettingsUpdate.includeAllNetworks(
            IncludeAllNetworksSettings(
                localNetworkSharingState: .on
            )
        )
        .apply(to: &updatedSettings)

        let tunnelSettingsStrategy = TunnelSettingsStrategy()
        XCTAssertEqual(
            tunnelSettingsStrategy.getReconnectionStrategy(
                oldSettings: currentSettings,
                newSettings: updatedSettings
            ), .hardReconnect)
    }
}
