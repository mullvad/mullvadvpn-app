//
//  DebugViewModel.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-02-27.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import Network
import PacketTunnelCore
import SwiftUI

@MainActor
protocol DebugViewModel: ObservableObject {
    typealias Item = (title: String, data: [String])

    var tunnelSettings: LatestTunnelSettings { get }
    var nwPathStatus: NWPath.Status { get }

    var connection: [Item] { get }
    var settings: [Item] { get }
}

class DebugViewModelImpl: DebugViewModel {
    var tunnelManager: TunnelManager
    var nwPathMonitor: NWPathMonitor
    var appPreferences: AppPreferencesDataSource
    var tunnelObserver: TunnelBlockObserver!

    var tunnelSettings: LatestTunnelSettings
    var tunnelStatus: TunnelStatus
    var nwPathStatus = NWPath.Status.unsatisfied

    @Published var connection = [Item]()
    @Published var settings = [Item]()

    init(
        tunnelManager: TunnelManager,
        nwPathMonitor: NWPathMonitor,
        appPreferences: AppPreferencesDataSource
    ) {
        self.tunnelManager = tunnelManager
        self.nwPathMonitor = nwPathMonitor
        self.appPreferences = appPreferences

        tunnelSettings = tunnelManager.settings
        tunnelStatus = tunnelManager.tunnelStatus

        refreshData()

        tunnelObserver = TunnelBlockObserver(
            didUpdateTunnelStatus: { _, status in
                self.tunnelStatus = status
                self.refreshData()
            },
            didUpdateDeviceState: { _, _, _ in },
            didUpdateTunnelSettings: { _, settings in
                self.tunnelSettings = settings
                self.refreshData()
            }
        )
        self.tunnelManager.addObserver(tunnelObserver)
    }

    // Arrange these functions to get the desired section order in the view.
    private func refreshData() {
        // Connection
        setRelays()
        setObfuscation()

        // Settings
        setRelaySettings()
        setMultihopSettings()
        setDaitaSettings()
        setObfuscationSettings()
        setQuantumResistanceSettings()
        setIncludeAllNetworksSettings()
        setMullvadDnsBlockers()
        setCustomDnsBlockers()
    }

    private func update(item: Item, in list: inout [Item]) {
        guard let index = (list.firstIndex { $0.title == item.title }) else {
            list.append(item)
            return
        }

        list.remove(at: index)
        list.insert(item, at: index)
    }
}

// MARK: Connection

extension DebugViewModelImpl {
    private func setRelays() {
        let entry = tunnelStatus.state.relays?.entry?.debugDescription ?? "-"
        let exit = tunnelStatus.state.relays?.exit.debugDescription ?? "-"

        update(
            item: (
                title: "Relays",
                data: [
                    "Entry: \(entry)",
                    "Exit: \(exit)",
                ]
            ), in: &connection
        )
    }

    private func setObfuscation() {
        update(
            item: (
                title: "Obfuscation",
                data: [
                    tunnelStatus.observedState.connectionState?.obfuscationMethod.description ?? "-"
                ]
            ), in: &connection
        )
    }
}

// MARK: Settings

extension DebugViewModelImpl {
    func setRelaySettings() {
        let entry = tunnelSettings.relayConstraints.entryLocations.value?.locations.first?.stringRepresentation ?? "-"
        let exit = tunnelSettings.relayConstraints.exitLocations.value?.locations.first?.stringRepresentation ?? "-"

        update(
            item: (
                title: "Relays",
                data: [
                    "Entry: \(entry)",
                    "Exit: \(exit)",
                ]
            ), in: &settings
        )
    }

    func setMullvadDnsBlockers() {
        let blockingOptions = tunnelSettings.dnsSettings.blockingOptions
        var dnsBlockers = [String]()

        if blockingOptions.contains(.blockAdvertising) {
            dnsBlockers.append("Advertising (\(DNSBlockingOptions.blockAdvertising.serverAddress!))")
        }
        if blockingOptions.contains(.blockTracking) {
            dnsBlockers.append("Tracking (\(DNSBlockingOptions.blockTracking.serverAddress!))")
        }
        if blockingOptions.contains(.blockMalware) {
            dnsBlockers.append("Malware (\(DNSBlockingOptions.blockMalware.serverAddress!))")
        }
        if blockingOptions.contains(.blockAdultContent) {
            dnsBlockers.append("Adult content (\(DNSBlockingOptions.blockAdultContent.serverAddress!))")
        }
        if blockingOptions.contains(.blockGambling) {
            dnsBlockers.append("Gambling (\(DNSBlockingOptions.blockGambling.serverAddress!))")
        }
        if blockingOptions.contains(.blockSocialMedia) {
            dnsBlockers.append("Social media (\(DNSBlockingOptions.blockSocialMedia.serverAddress!))")
        }

        update(
            item: (
                title: "Mullvad DNS blockers",
                data: dnsBlockers.isEmpty ? ["-"] : dnsBlockers
            ), in: &settings
        )
    }

    func setCustomDnsBlockers() {
        let customAddresses = tunnelSettings.dnsSettings.customDNSDomains.map { $0.debugDescription }

        update(
            item: (
                title: "Custom DNS blockers",
                data: customAddresses.isEmpty ? ["-"] : customAddresses
            ), in: &settings
        )
    }

    func setObfuscationSettings() {
        let method = tunnelSettings.wireGuardObfuscation.state.description
        let udpTcpPort = tunnelSettings.wireGuardObfuscation.udpOverTcpPort.description
        let shadowSocksPort = tunnelSettings.wireGuardObfuscation.shadowsocksPort.description

        update(
            item: (
                title: "Obfuscation",
                data: [
                    "Method: \(method)",
                    "UDP over TCP port: \(udpTcpPort)",
                    "Shadowsocks port: \(shadowSocksPort)",
                ]
            ), in: &settings
        )
    }

    func setQuantumResistanceSettings() {
        update(
            item: (
                title: "Quantum resistance",
                data: [
                    tunnelSettings.tunnelQuantumResistance.isEnabled ? "Enabled" : "Disabled"
                ]
            ), in: &settings
        )
    }

    func setMultihopSettings() {
        update(
            item: (
                title: "Multihop",
                data: [
                    tunnelSettings.tunnelMultihopState.isEnabled ? "Enabled" : "Disabled"
                ]
            ), in: &settings
        )
    }

    func setDaitaSettings() {
        let daitaIsEnabled = tunnelSettings.daita.daitaState.isEnabled ? "Enabled" : "Disabled"
        let directOnlyIsEnabled = tunnelSettings.daita.directOnlyState.isEnabled ? "Enabled" : "Disabled"

        update(
            item: (
                title: "DAITA",
                data: [
                    "DAITA: \(daitaIsEnabled)",
                    "Direct only: \(directOnlyIsEnabled)",
                ]
            ), in: &settings
        )
    }

    func setIncludeAllNetworksSettings() {
        let includeAllNetworksIsEnabled =
            tunnelSettings.includeAllNetworks.includeAllNetworksIsEnabled ? "Enabled" : "Disabled"
        let localNetworkSharingIsEnabled =
            tunnelSettings.includeAllNetworks.localNetworkSharingIsEnabled ? "Enabled" : "Disabled"
        let consent = appPreferences.includeAllNetworksConsent ? "True" : "False"

        update(
            item: (
                title: "Force all apps",
                data: [
                    "Force all apps: \(includeAllNetworksIsEnabled)",
                    "Local network sharing: \(localNetworkSharingIsEnabled)",
                    "Consent: \(consent)",
                ]
            ), in: &settings
        )
    }
}

// MARK: Mock

class MockDebugViewModel: DebugViewModel {
    var tunnelSettings: LatestTunnelSettings = LatestTunnelSettings()
    var nwPathStatus: NWPath.Status = NWPath.Status.unsatisfied

    // Connection
    var connection = [
        (
            title: "Relays",
            data: [
                "Entry: Sweden, Gothenburg, se-got-001",
                "Exit: Sweden, Gothenburg, se-got-002",
            ]
        ),
        (title: "Obfuscation", data: [""]),
    ]

    // Settings
    var settings = [
        (title: "Relays", data: ["Entry: se-se-got-se-got-001", "Exit: se-se-got-se-got-001"]),
        (
            title: "Mullvad DNS blockers",
            data: [
                "Advertising (100.64.0.1)",
                "Tracking (100.64.0.2)",
                "Social Media (100.64.0.3)",
            ]
        ),
        (title: "Custom DNS blockers", data: ["192.168.1.1", "192.168.1.2"]),
        (title: "Obfuscation", data: ["QUIC", "UDP over TCP port: 53", "Shadowsocks port: 53"]),
        (title: "Quantum resistance", data: ["Enabled"]),
        (title: "Multihop", data: ["Disabled"]),
        (title: "DAITA", data: ["DAITA: Enabled", "Direct only: Disabled"]),
        (
            title: "Force all apps",
            data: [
                "Force all apps: Disabled",
                "Local network sharing: Disabled",
                "Consent: False",
            ]
        ),
    ]
}
