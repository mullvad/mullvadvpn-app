//
//  PostQuantumKeyNegotiator.swift
//  PacketTunnel
//
//  Created by Marco Nikic on 2024-02-16.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension
import TalpidTunnelConfigClientProxy
import WireGuardKitTypes

/**
 Attempt to start the asynchronous process of key negotiation. Returns true if successfully started, false if failed.
 */
public class PostQuantumKeyNegotiator {
    public init() {}

    var cancelToken: PostQuantumCancelToken?

    public func startNegotiation(
        gatewayIP: IPv4Address,
        devicePublicKey: PublicKey,
        presharedKey: PrivateKey,
        packetTunnel: NEPacketTunnelProvider,
        tcpConnection: NWTCPConnection
    ) -> Bool {
        let packetTunnelPointer = Unmanaged.passUnretained(packetTunnel).toOpaque()
        let opaqueConnection = Unmanaged.passUnretained(tcpConnection).toOpaque()
        var cancelToken = PostQuantumCancelToken()

        let result = negotiate_post_quantum_key(
            devicePublicKey.rawValue.map { $0 },
            presharedKey.rawValue.map { $0 },
            packetTunnelPointer,
            opaqueConnection,
            &cancelToken
        )
        guard result == 0 else {
            return false
        }
        self.cancelToken = cancelToken
        return true
    }

    public func cancelKeyNegotiation() {
        guard var cancelToken else { return }
        cancel_post_quantum_key_exchange(&cancelToken)
    }

    deinit {
        guard var cancelToken else { return }
        drop_post_quantum_key_exchange_token(&cancelToken)
    }
}
