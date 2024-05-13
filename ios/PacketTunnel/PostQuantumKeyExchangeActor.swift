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

class PostQuantumKeyExchangeActor {
    struct Negotiation {
        var negotiator: PostQuantumKeyNegotiator
        var inTunnelTCPConnection: NWTCPConnection
        var tcpConnectionObserver: NSKeyValueObservation

        func cancel() {
            negotiator.cancelKeyNegotiation()
            tcpConnectionObserver.invalidate()
            inTunnelTCPConnection.cancel()
        }
    }

    unowned let packetTunnel: PacketTunnelProvider
    private var negotiation: Negotiation?

    // Callback in the event of the negotiation failing on startup
    var onFailure: () -> Void

    init(packetTunnel: PacketTunnelProvider, onFailure: @escaping (() -> Void)) {
        self.packetTunnel = packetTunnel
        self.onFailure = onFailure
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
        let negotiator = PostQuantumKeyNegotiator()

        let gatewayAddress = "10.64.0.1"
        let IPv4Gateway = IPv4Address(gatewayAddress)!
        let endpoint = NWHostEndpoint(hostname: gatewayAddress, port: "1337")
        let inTunnelTCPConnection = createTCPConnection(endpoint)

        // This will become the new private key of the device
        let ephemeralSharedKey = PrivateKey()

        let tcpConnectionObserver = inTunnelTCPConnection.observe(\.isViable, options: [
            .initial,
            .new,
        ]) { [weak self] observedConnection, _ in
            guard let self, observedConnection.isViable else { return }
            self.negotiation?.tcpConnectionObserver.invalidate()
            if !negotiator.startNegotiation(
                gatewayIP: IPv4Gateway,
                devicePublicKey: privateKey.publicKey,
                presharedKey: ephemeralSharedKey,
                packetTunnel: packetTunnel,
                tcpConnection: inTunnelTCPConnection
            ) {
                self.negotiation = nil
                self.onFailure()
            }
        }
        negotiation = Negotiation(
            negotiator: negotiator,
            inTunnelTCPConnection: inTunnelTCPConnection,
            tcpConnectionObserver: tcpConnectionObserver
        )
    }

    func endCurrentNegotiation() {
        negotiation?.cancel()
        negotiation = nil
    }
}
