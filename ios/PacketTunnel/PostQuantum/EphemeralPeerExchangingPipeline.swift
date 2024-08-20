//
//  PostQuantumKeyExchangingPipeline.swift
//  PacketTunnel
//
//  Created by Mojgan on 2024-07-15.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadRustRuntime
import MullvadSettings
import PacketTunnelCore
import WireGuardKitTypes

final public class EphemeralPeerExchangingPipeline {
    let keyExchanger: EphemeralPeerExchangeActorProtocol
    let onUpdateConfiguration: (EphemeralPeerNegotiationState) -> Void
    let onFinish: () -> Void

    private var ephemeralPeerExchanger: EphemeralPeerExchangingProtocol!

    public init(
        _ keyExchanger: EphemeralPeerExchangeActorProtocol,
        onUpdateConfiguration: @escaping (EphemeralPeerNegotiationState) -> Void,
        onFinish: @escaping () -> Void
    ) {
        self.keyExchanger = keyExchanger
        self.onUpdateConfiguration = onUpdateConfiguration
        self.onFinish = onFinish
    }

    public func startNegotiation(_ connectionState: ObservedConnectionState, privateKey: PrivateKey) {
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
        ephemeralPeerExchanger.start()
    }

    public func receivePostQuantumKey(_ key: PreSharedKey, ephemeralKey: PrivateKey) {
        ephemeralPeerExchanger.receivePostQuantumKey(key, ephemeralKey: ephemeralKey)
    }

    public func receiveEphemeralPeerPrivateKey(_ ephemeralPeerPrivateKey: PrivateKey) {
        ephemeralPeerExchanger.receiveEphemeralPeerPrivateKey(ephemeralPeerPrivateKey)
    }
}
