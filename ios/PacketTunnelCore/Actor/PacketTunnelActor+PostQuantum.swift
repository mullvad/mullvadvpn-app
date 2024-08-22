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
    internal func tryStartEphemeralPeerNegotiation(
        withSettings settings: Settings,
        nextRelays: NextRelays,
        reason: ActorReconnectReason
    ) async throws {
        if let connectionState = try obfuscateConnection(nextRelays: nextRelays, settings: settings, reason: reason) {
            let activeKey = activeKey(from: connectionState, in: settings)
            state = .negotiatingEphemeralPeer(connectionState, activeKey)
        }
    }

    /**
     Called on receipt of the new PQ-negotiated key, to reconnect to the relay, in PQ-secure mode.
     */
    internal func connectWithEphemeralPeer() async {
        guard let connectionData = state.connectionData else {
            logger.error("Could not create connection state in PostQuantumConnect")
            eventChannel.send(.reconnect(.current))
            return
        }

        stopDefaultPathObserver()

        state = .connecting(connectionData)

        // Resume tunnel monitoring and use IPv4 gateway as a probe address.
        tunnelMonitor.start(probeAddress: connectionData.selectedRelays.exit.endpoint.ipv4Gateway)
        // Restart default path observer and notify the observer with the current path that might have changed while
        // path observer was paused.
        startDefaultPathObserver(notifyObserverWithCurrentPath: false)
    }

    /**
     Called to reconfigure the tunnel after each ephemeral peer negotiation.
     */
    internal func updateEphemeralPeerNegotiationState(configuration: EphemeralPeerNegotiationState) async throws {
        /**
         The obfuscater needs to be restarted every time a new tunnel configuration is being used,
         because the obfuscation may be tied to a specific UDP session, as is the case for udp2tcp.
         */
        let settings: Settings = try settingsReader.read()
        guard let connectionData = try obfuscateConnection(
            nextRelays: .current,
            settings: settings,
            reason: .userInitiated
        ) else {
            logger.error("Tried to update ephemeral peer negotiation in invalid state: \(state.name)")
            return
        }

        var daitaConfiguration: DaitaConfiguration?
        if settings.daita.state.isEnabled {
            let maybeNot = Maybenot()
            daitaConfiguration = DaitaConfiguration(
                machines: maybeNot.machines,
                maxEvents: maybeNot.maximumEvents,
                maxActions: maybeNot.maximumActions
            )
        }

        switch configuration {
        case let .single(hop):
            let exitConfiguration = try ConfigurationBuilder(
                privateKey: hop.configuration.privateKey,
                interfaceAddresses: settings.interfaceAddresses,
                dns: settings.dnsServers,
                endpoint: connectionData.connectedEndpoint,
                allowedIPs: hop.configuration.allowedIPs,
                preSharedKey: hop.configuration.preSharedKey
            ).makeConfiguration()

            try await tunnelAdapter.start(configuration: exitConfiguration, daita: daitaConfiguration)

        case let .multi(firstHop, secondHop):
            let entryConfiguration = try ConfigurationBuilder(
                privateKey: firstHop.configuration.privateKey,
                interfaceAddresses: settings.interfaceAddresses,
                dns: settings.dnsServers,
                endpoint: connectionData.connectedEndpoint,
                allowedIPs: firstHop.configuration.allowedIPs,
                preSharedKey: firstHop.configuration.preSharedKey
            ).makeConfiguration()

            let exitConfiguration = try ConfigurationBuilder(
                privateKey: secondHop.configuration.privateKey,
                interfaceAddresses: settings.interfaceAddresses,
                dns: settings.dnsServers,
                endpoint: secondHop.relay.endpoint,
                allowedIPs: secondHop.configuration.allowedIPs,
                preSharedKey: secondHop.configuration.preSharedKey
            ).makeConfiguration()

            try await tunnelAdapter.startMultihop(
                entryConfiguration: entryConfiguration, exitConfiguration: exitConfiguration, daita: daitaConfiguration
            )
        }
    }
}
