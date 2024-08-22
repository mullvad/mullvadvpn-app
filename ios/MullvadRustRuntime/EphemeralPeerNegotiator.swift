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

// swiftlint:disable function_parameter_count
public protocol EphemeralPeerNegotiating {
    func startNegotiation(
        gatewayIP: IPv4Address,
        devicePublicKey: PublicKey,
        presharedKey: PrivateKey,
        peerReceiver: any TunnelProvider,
        tcpConnection: NWTCPConnection,
        peerExchangeTimeout: Duration,
        enablePostQuantum: Bool,
        enableDaita: Bool
    ) -> Bool

    func cancelKeyNegotiation()

    init()
}

/// Requests an ephemeral peer asynchronously.
public class EphemeralPeerNegotiator: EphemeralPeerNegotiating {
    required public init() {}

    var cancelToken: EphemeralPeerCancelToken?

    public func startNegotiation(
        gatewayIP: IPv4Address,
        devicePublicKey: PublicKey,
        presharedKey: PrivateKey,
        peerReceiver: any TunnelProvider,
        tcpConnection: NWTCPConnection,
        peerExchangeTimeout: Duration,
        enablePostQuantum: Bool,
        enableDaita: Bool
    ) -> Bool {
        // swiftlint:disable:next force_cast
        let ephemeralPeerReceiver = Unmanaged.passUnretained(peerReceiver as! EphemeralPeerReceiver)
            .toOpaque()
        let opaqueConnection = Unmanaged.passUnretained(tcpConnection).toOpaque()
        var cancelToken = EphemeralPeerCancelToken()

        let result = request_ephemeral_peer(
            devicePublicKey.rawValue.map { $0 },
            presharedKey.rawValue.map { $0 },
            ephemeralPeerReceiver,
            opaqueConnection,
            &cancelToken,
            UInt64(peerExchangeTimeout.timeInterval),
            enablePostQuantum,
            enableDaita
        )
        guard result == 0 else {
            return false
        }
        self.cancelToken = cancelToken
        return true
    }

    public func cancelKeyNegotiation() {
        guard var cancelToken else { return }
        cancel_ephemeral_peer_exchange(&cancelToken)
    }

    deinit {
        guard var cancelToken else { return }
        drop_ephemeral_peer_exchange_token(&cancelToken)
    }
}

// swiftlint:enable function_parameter_count
