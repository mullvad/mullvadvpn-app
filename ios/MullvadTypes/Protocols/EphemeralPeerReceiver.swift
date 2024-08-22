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
    unowned let tunnelProvider: NEPacketTunnelProvider

    public init(tunnelProvider: NEPacketTunnelProvider) {
        self.tunnelProvider = tunnelProvider
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
