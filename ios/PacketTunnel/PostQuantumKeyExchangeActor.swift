//
//  PostQuantumKeyExchangeActor.swift
//  PacketTunnel
//
//  Created by Marco Nikic on 2024-04-12.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadPostQuantum
import NetworkExtension
import WireGuardKitTypes

// not needed? as we have the PacketTunnelProvider
// typealias InTunnelTCPConnectionCreator = (NWHostEndpoint) -> NWTCPConnection

actor PostQuantumKeyExchangeActor {
//    let createNetworkConnection: InTunnelTCPConnectionCreator
    unowned let packetTunnel: PacketTunnelProvider
    private var negotiator: PostQuantumKeyNegotiator?
    private var inTunnelTCPConnection: NWTCPConnection?
    private var tcpConnectionObserver: NSKeyValueObservation?

    init(packetTunnel: PacketTunnelProvider /* , createNetworkConnection: @escaping InTunnelTCPConnectionCreator */ ) {
        self.packetTunnel = packetTunnel
    }

    private func createTCPConnection(_ gatewayEndpoint: NWHostEndpoint) -> NWTCPConnection {
        self.packetTunnel.createTCPConnectionThroughTunnel(
            to: gatewayEndpoint,
            enableTLS: false,
            tlsParameters: nil,
            delegate: nil
        )
    }

    func startNegotiation(with privateKey: PrivateKey) {
        negotiator = PostQuantumKeyNegotiator()

        let gatewayAddress = "10.64.0.1"
        let IPv4Gateway = IPv4Address(gatewayAddress)!
        let endpoint = NWHostEndpoint(hostname: gatewayAddress, port: "1337")
        inTunnelTCPConnection = createTCPConnection(endpoint)

        let ephemeralSharedKey = PrivateKey() // This will become the new private key of the device

        tcpConnectionObserver = inTunnelTCPConnection!.observe(\.isViable, options: [
            .initial,
            .new,
        ]) { [weak self] observedConnection, _ in
            guard let self, observedConnection.isViable else { return }
            negotiator!.negotiateKey(
                gatewayIP: IPv4Gateway,
                devicePublicKey: privateKey.publicKey,
                presharedKey: ephemeralSharedKey,
                packetTunnel: packetTunnel,
                tcpConnection: inTunnelTCPConnection!
            )
            self.tcpConnectionObserver!.invalidate()
        }
    }
}
