//
//  PostQuantumKeyExchangingPipeline.swift
//  PacketTunnel
//
//  Created by Mojgan on 2024-07-15.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadRustRuntime
import MullvadSettings
import MullvadTypes
import PacketTunnelCore
import WireGuardKitTypes

final public class EphemeralPeerExchangingPipeline {
    let keyExchanger: EphemeralPeerExchangeActorProtocol
    let onUpdateConfiguration: (EphemeralPeerNegotiationState) async -> Void
    let onFinish: () -> Void

    private var ephemeralPeerExchanger: EphemeralPeerExchangingProtocol!

    public init(
        _ keyExchanger: EphemeralPeerExchangeActorProtocol,
        onUpdateConfiguration: @escaping (EphemeralPeerNegotiationState) async -> Void,
        onFinish: @escaping () -> Void
    ) {
        self.keyExchanger = keyExchanger
        self.onUpdateConfiguration = onUpdateConfiguration
        self.onFinish = onFinish
    }

    public func startNegotiation(_ connectionState: ObservedConnectionState, privateKey: PrivateKey) async {
        keyExchanger.reset()
        let entryPeer = connectionState.selectedRelays.entry
        let exitPeer = connectionState.selectedRelays.exit
        let enablePostQuantum = connectionState.isPostQuantum
        let enableDaita = connectionState.isDaitaEnabled
        if let entryPeer {
            ephemeralPeerExchanger = MultiHopEphemeralPeerExchanger(
                entry: entryPeer,
                exit: exitPeer,
                devicePrivateKey: privateKey,
                keyExchanger: keyExchanger,
                enablePostQuantum: enablePostQuantum,
                enableDaita: enableDaita,
                onUpdateConfiguration: self.onUpdateConfiguration,
                onFinish: onFinish
            )
        } else {
            ephemeralPeerExchanger = SingleHopEphemeralPeerExchanger(
                exit: exitPeer,
                devicePrivateKey: privateKey,
                keyExchanger: keyExchanger,
                enablePostQuantum: enablePostQuantum,
                enableDaita: enableDaita,
                onUpdateConfiguration: self.onUpdateConfiguration,
                onFinish: onFinish
            )
        }
        await ephemeralPeerExchanger.start()
    }

    public func receivePostQuantumKey(
        _ key: PreSharedKey,
        ephemeralKey: PrivateKey,
        daitaParameters: DaitaV2Parameters?
    ) async {
        await ephemeralPeerExchanger.receivePostQuantumKey(
            key,
            ephemeralKey: ephemeralKey,
            daitaParameters: daitaParameters
        )
    }

    public func receiveEphemeralPeerPrivateKey(
        _ ephemeralPeerPrivateKey: PrivateKey,
        daitaParameters: DaitaV2Parameters?
    ) async {
        await ephemeralPeerExchanger.receiveEphemeralPeerPrivateKey(
            ephemeralPeerPrivateKey,
            daitaParameters: daitaParameters
        )
    }
}
