//
//  PacketTunnelActor+PostQuantum.swift
//  PacketTunnelCore
//
//  Created by Andrew Bulhak on 2024-05-13.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import WireGuardKitTypes

extension PacketTunnelActor {
    /**
     Attempt to start the process of negotiating a post-quantum secure key, setting up an initial
     connection restricted to the negotiation host and entering the negotiating state.
     */
    internal func tryStartPostQuantumNegotiation(
        withSettings settings: Settings,
        nextRelay: NextRelay,
        reason: ReconnectReason
    ) async throws {
        if let connectionState = try makeConnectionState(nextRelay: nextRelay, settings: settings, reason: reason) {
            let selectedEndpoint = connectionState.selectedRelay.endpoint
            let activeKey = activeKey(from: connectionState, in: settings)

            let configurationBuilder = ConfigurationBuilder(
                privateKey: activeKey,
                interfaceAddresses: settings.interfaceAddresses,
                dns: settings.dnsServers,
                endpoint: selectedEndpoint,
                allowedIPs: [
                    IPAddressRange(from: "10.64.0.1/32")!,
                ]
            )

            try await tunnelAdapter.start(configuration: configurationBuilder.makeConfiguration())
            state = .negotiatingPostQuantumKey(connectionState, activeKey)
        }
    }

    /**
     Called on receipt of the new PQ-negotiated key, to reconnect to the relay, in PQ-secure mode.
     */
    internal func postQuantumConnect(with key: PreSharedKey, privateKey: PrivateKey) async {
        guard
            // It is important to select the same relay that was saved in the connection state as the key negotiation happened with this specific relay.
            let selectedRelay = state.connectionData?.selectedRelay,
            let settings: Settings = try? settingsReader.read(),
            let connectionState = try? obfuscateConnection(
                nextRelay: .preSelected(selectedRelay),
                settings: settings,
                reason: .userInitiated
            )
        else {
            logger.error("Could not create connection state in PostQuantumConnect")
            setErrorState(reason: .unknown)
            return
        }

        let configurationBuilder = ConfigurationBuilder(
            privateKey: privateKey,
            interfaceAddresses: settings.interfaceAddresses,
            dns: settings.dnsServers,
            endpoint: connectionState.connectedEndpoint,
            allowedIPs: [
                IPAddressRange(from: "0.0.0.0/0")!,
                IPAddressRange(from: "::/0")!,
            ],
            preSharedKey: key
        )
        stopDefaultPathObserver()

        state = .connecting(connectionState)

        try? await tunnelAdapter.start(configuration: configurationBuilder.makeConfiguration())
        // Resume tunnel monitoring and use IPv4 gateway as a probe address.
        tunnelMonitor.start(probeAddress: connectionState.selectedRelay.endpoint.ipv4Gateway)
        // Restart default path observer and notify the observer with the current path that might have changed while
        // path observer was paused.
        startDefaultPathObserver(notifyObserverWithCurrentPath: false)
    }
}
