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
import PacketTunnelCore
import TalpidTunnelConfigClientProxy
import WireGuardKitTypes

// swiftlint:disable function_parameter_count
public protocol PostQuantumKeyNegotiation {
    func startNegotiation(
        gatewayIP: IPv4Address,
        devicePublicKey: PublicKey,
        presharedKey: PrivateKey,
        packetTunnel: any TunnelProvider,
        tcpConnection: NWTCPConnection,
        postQuantumKeyExchangeTimeout: Duration
    ) -> Bool

    func cancelKeyNegotiation()

    init()
}

/**
 Attempt to start the asynchronous process of key negotiation. Returns true if successfully started, false if failed.
 */
public class PostQuantumKeyNegotiator: PostQuantumKeyNegotiation {
    required public init() {}

    var cancelToken: PostQuantumCancelToken?

    public func startNegotiation(
        gatewayIP: IPv4Address,
        devicePublicKey: PublicKey,
        presharedKey: PrivateKey,
        packetTunnel: any TunnelProvider,
        tcpConnection: NWTCPConnection,
        postQuantumKeyExchangeTimeout: Duration
    ) -> Bool {
        // swiftlint:disable:next force_cast
        let packetTunnelPointer = Unmanaged.passUnretained(packetTunnel as! NEPacketTunnelProvider).toOpaque()
        let opaqueConnection = Unmanaged.passUnretained(tcpConnection).toOpaque()
        var cancelToken = PostQuantumCancelToken()

        let result = negotiate_post_quantum_key(
            devicePublicKey.rawValue.map { $0 },
            presharedKey.rawValue.map { $0 },
            packetTunnelPointer,
            opaqueConnection,
            &cancelToken,
            UInt64(postQuantumKeyExchangeTimeout.timeInterval)
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

// swiftlint:enable function_parameter_count
