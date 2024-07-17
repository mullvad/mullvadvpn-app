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

public class PostQuantumKeyReceiver: PostQuantumKeyReceiving, TunnelProvider {
    unowned let tunnelProvider: NEPacketTunnelProvider

    public init(tunnelProvider: NEPacketTunnelProvider) {
        self.tunnelProvider = tunnelProvider
    }

    // MARK: - PostQuantumKeyReceiving

    public func receivePostQuantumKey(_ key: PreSharedKey, ephemeralKey: PrivateKey) {
        guard let receiver = tunnelProvider as? PostQuantumKeyReceiving else { return }
        receiver.receivePostQuantumKey(key, ephemeralKey: ephemeralKey)
    }

    public func keyExchangeFailed() {
        guard let receiver = tunnelProvider as? PostQuantumKeyReceiving else { return }
        receiver.keyExchangeFailed()
    }

    // MARK: - TunnelProvider

    public func createTCPConnectionThroughTunnel(
        to remoteEndpoint: NWEndpoint,
        enableTLS: Bool,
        tlsParameters TLSParameters: NWTLSParameters?,
        delegate: Any?
    ) -> NWTCPConnection {
        tunnelProvider.createTCPConnectionThroughTunnel(
            to: remoteEndpoint,
            enableTLS: enableTLS,
            tlsParameters: TLSParameters,
            delegate: delegate
        )
    }
}
