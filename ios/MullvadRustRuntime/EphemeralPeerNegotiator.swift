//
//  PostQuantumKeyNegotiator.swift
//  PacketTunnel
//
//  Created by Marco Nikic on 2024-02-16.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import NetworkExtension
import WireGuardKitTypes

public protocol EphemeralPeerNegotiating {
    func startNegotiation(
        devicePublicKey: PublicKey,
        presharedKey: PrivateKey,
        peerReceiver: any TunnelProvider,
        ephemeralPeerParams: EphemeralPeerParameters
    ) -> Bool

    func cancelKeyNegotiation()

    init()
}

/// Requests an ephemeral peer asynchronously.
public class EphemeralPeerNegotiator: EphemeralPeerNegotiating {
    required public init() {}

    var cancelToken: OpaquePointer?

    public func startNegotiation(
        devicePublicKey: PublicKey,
        presharedKey: PrivateKey,
        peerReceiver: any TunnelProvider,
        ephemeralPeerParams: EphemeralPeerParameters
    ) -> Bool {
        // swiftlint:disable:next force_cast
        let ephemeralPeerReceiver = Unmanaged.passUnretained(peerReceiver as! EphemeralPeerReceiver)
            .toOpaque()

        guard let tunnelHandle = try? peerReceiver.tunnelHandle() else {
            return false
        }

        let cancelToken = request_ephemeral_peer(
            devicePublicKey.rawValue.map { $0 },
            presharedKey.rawValue.map { $0 },
            ephemeralPeerReceiver,
            tunnelHandle,
            ephemeralPeerParams
        )
        guard let cancelToken else {
            return false
        }
        self.cancelToken = cancelToken
        return true
    }

    public func cancelKeyNegotiation() {
        guard let cancelToken else { return }
        cancel_ephemeral_peer_exchange(cancelToken)
        self.cancelToken = nil
    }

    deinit {
        guard let cancelToken else { return }
        drop_ephemeral_peer_exchange_token(cancelToken)
    }
}
