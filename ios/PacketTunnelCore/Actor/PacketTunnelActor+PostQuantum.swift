//
//  PacketTunnelActor+PostQuantum.swift
//  PacketTunnelCore
//
//  Created by Andrew Bulhak on 2024-05-13.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
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
        if let connectionState = try obfuscateConnection(
            nextRelays: nextRelays,
            settings: settings,
            ephemeralPeerKey: nil,
            reason: reason
        ) {
            let activeKey = activeKey(from: connectionState, in: settings)
            state = .negotiatingEphemeralPeer(connectionState, activeKey)
        }
    }

    /**
     Called on receipt of the new PQ-negotiated key, to reconnect to the relay, in PQ-secure mode.
     */
    internal func connectWithEphemeralPeer() async {
        // Don't reconnect if we're in error state - stay in error state
        // The error occurred during configuration, not negotiation
        if case .error = state {
            logger.error("Cannot connect with ephemeral peer: actor is in error state. Staying in error state.")
            return
        }

        guard let connectionData = state.connectionData else {
            logger.error("Could not create connection state in PostQuantumConnect")
            eventChannel.send(.reconnect(.current))
            return
        }

        state = .connecting(connectionData)

        // Resume tunnel monitoring and use IPv4 gateway as a probe address.
        tunnelMonitor.start(probeAddress: connectionData.selectedRelays.exit.endpoint.ipv4Gateway)
    }

    /**
     Called to reconfigure the tunnel after each ephemeral peer negotiation.
     */
    internal func updateEphemeralPeerNegotiationState(configuration: EphemeralPeerNegotiationState) async throws {
        let ephemeralPeerKey = configuration.ephemeralPeerKeys.entry ?? configuration.ephemeralPeerKeys.exit
        let settings: Settings = try settingsReader.read()

        /**
         The obfuscator needs to be restarted every time a new tunnel configuration is being used,
         because the obfuscation may be tied to a specific UDP session, as is the case for udp2tcp.
         */
        guard
            let connectionData = try obfuscateConnection(
                nextRelays: .current,
                settings: settings,
                ephemeralPeerKey: ephemeralPeerKey,
                reason: .userInitiated
            )
        else {
            logger.error("Tried to update ephemeral peer negotiation in invalid state: \(state.name)")
            return
        }

        var daitaConfiguration: DaitaConfiguration?

        switch configuration {
        case let .single(hop):
            let exitConfiguration = try ConnectionConfigurationBuilder(
                type: .ephemeral(.single(hop)),
                settings: settings,
                connectionData: connectionData
            ).make().exitConfiguration
            if settings.daita.daitaState.isEnabled, let daitaSettings = hop.configuration.daitaParameters {
                daitaConfiguration = DaitaConfiguration(daita: daitaSettings)
            }
            try await tunnelAdapter.apply(settings: exitConfiguration.asTunnelSettings())
            try await tunnelAdapter.start(configuration: exitConfiguration, daita: daitaConfiguration)

        case let .multi(firstHop, secondHop):
            let connectionConfiguration = try ConnectionConfigurationBuilder(
                type: .ephemeral(.multi(entry: firstHop, exit: secondHop)),
                settings: settings,
                connectionData: connectionData
            ).make()

            if settings.daita.daitaState.isEnabled, let daitaSettings = firstHop.configuration.daitaParameters {
                daitaConfiguration = DaitaConfiguration(daita: daitaSettings)
            }

            try await tunnelAdapter
                .apply(
                    settings: connectionConfiguration.exitConfiguration
                        .asTunnelSettings()
                )
            try await tunnelAdapter.startMultihop(
                entryConfiguration: connectionConfiguration.entryConfiguration,
                exitConfiguration: connectionConfiguration.exitConfiguration, daita: daitaConfiguration
            )
        }
    }
}

extension DaitaConfiguration {
    init(daita: DaitaV2Parameters) {
        self = DaitaConfiguration(
            machines: daita.machines,
            maxEvents: daita.maximumEvents,
            maxActions: daita.maximumActions,
            maxPadding: daita.maximumPadding,
            maxBlocking: daita.maximumBlocking
        )
    }
}
