//
//  PostQuantumKeyReceiver.swift
//  MullvadTypes
//
//  Created by Marco Nikic on 2024-07-04.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension
import WireGuardKitTypes

public class EphemeralPeerReceiver: EphemeralPeerReceiving, TunnelProvider {
    public func tunnelHandle() throws -> Int32 {
        try tunnelProvider.tunnelHandle()
    }

    public func wgFuncs() -> WgFuncPointers {
        tunnelProvider.wgFuncs()
    }

    unowned let tunnelProvider: any TunnelProvider
    let keyReceiver: any EphemeralPeerReceiving

    public init(tunnelProvider: TunnelProvider, keyReceiver: any EphemeralPeerReceiving) {
        self.tunnelProvider = tunnelProvider
        self.keyReceiver = keyReceiver
    }

    // MARK: - EphemeralPeerReceiving

    public func receivePostQuantumKey(_ key: PreSharedKey, ephemeralKey: PrivateKey) {
        guard let receiver = tunnelProvider as? EphemeralPeerReceiving else { return }
        receiver.receivePostQuantumKey(key, ephemeralKey: ephemeralKey)
    }

    public func receiveEphemeralPeerPrivateKey(_ ephemeralPeerPrivateKey: PrivateKey) {
        guard let receiver = tunnelProvider as? EphemeralPeerReceiving else { return }
        receiver.receiveEphemeralPeerPrivateKey(ephemeralPeerPrivateKey)
    }

    public func ephemeralPeerExchangeFailed() {
        guard let receiver = tunnelProvider as? EphemeralPeerReceiving else { return }
        receiver.ephemeralPeerExchangeFailed()
    }
}
