//
//  PostQuantumKeyNegotiatior.swift
//  PacketTunnel
//
//  Created by Marco Nikic on 2024-02-16.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension
import TalpidTunnelConfigClientProxy
import WireGuardKitTypes

public class PostQuantumKeyNegotiatior {
    private var cancellationToken: UnsafeRawPointer!
    public init() {}

    public func negotiateKey(
        gatewayIP: IPv4Address,
        devicePublicKey: PublicKey,
        presharedKey: PublicKey,
        packetTunnel: NEPacketTunnelProvider,
        tcpConnection: NWTCPConnection
    ) {
        let packetTunnelPointer = Unmanaged.passUnretained(packetTunnel).toOpaque()
        let opaqueConnection = Unmanaged.passUnretained(tcpConnection).toOpaque()

        // TODO: Any non 0 return is considered a failure, and should be handled gracefully
        let token = negotiate_post_quantum_key(
            devicePublicKey.rawValue.map { $0 },
            presharedKey.rawValue.map { $0 },
            packetTunnelPointer,
            opaqueConnection
        )
        guard let token else {
            // Handle failure here
            return
        }

        cancellationToken = token
    }

    public func cancelKeyNegotiation() {
        if let cancellationToken, cancellationToken.hashValue != 0.hashValue {
            cancel_post_quantum_key_exchange(cancellationToken)
        }
    }
}
