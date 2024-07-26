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

final public class PostQuantumKeyExchangingPipeline {
    let keyExchanger: PostQuantumKeyExchangeActorProtocol
    let onUpdateConfiguration: (PostQuantumNegotiationState) -> Void
    let onFinish: () -> Void

    private var postQuantumKeyExchanging: PostQuantumKeyExchangingProtocol!

    public init(
        _ keyExchanger: PostQuantumKeyExchangeActorProtocol,
        onUpdateConfiguration: @escaping (PostQuantumNegotiationState) -> Void,
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
        if let entryPeer {
            postQuantumKeyExchanging = MultiHopPostQuantumKeyExchanging(
                entry: entryPeer,
                exit: exitPeer,
                devicePrivateKey: privateKey,
                keyExchanger: keyExchanger,
                onUpdateConfiguration: self.onUpdateConfiguration,
                onFinish: onFinish
            )
        } else {
            postQuantumKeyExchanging = SingleHopPostQuantumKeyExchanging(
                exit: exitPeer,
                devicePrivateKey: privateKey,
                keyExchanger: keyExchanger,
                onUpdateConfiguration: self.onUpdateConfiguration,
                onFinish: onFinish
            )
        }
        postQuantumKeyExchanging.start()
    }

    public func receivePostQuantumKey(_ key: PreSharedKey, ephemeralKey: PrivateKey) {
        postQuantumKeyExchanging.receivePostQuantumKey(key, ephemeralKey: ephemeralKey)
    }
}
