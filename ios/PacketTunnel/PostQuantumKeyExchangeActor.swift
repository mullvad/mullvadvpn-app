//
//  PostQuantumKeyExchangeActor.swift
//  PacketTunnel
//
//  Created by Marco Nikic on 2024-04-12.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadPostQuantum
import MullvadREST
import MullvadTypes
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
    private var timer: DispatchSourceTimer?
    private var keyExchangeRetriesIterator = REST.RetryStrategy.postQuantumKeyExchange.makeDelayIterator()

    // Callback in the event of the negotiation failing on startup
    var onFailure: () -> Void

    init(
        packetTunnel: PacketTunnelProvider,
        onFailure: @escaping (() -> Void)
    ) {
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

    /// Starts a new key exchange.
    ///
    /// Any ongoing key negotiation is stopped before starting a new one.
    /// An exponential backoff timer is used to stop the exchange if it takes too long,
    /// or if the TCP connection takes too long to become ready.
    /// It is reset after every successful key exchange.
    ///
    /// - Parameter privateKey: The device's current private key
    func startNegotiation(with privateKey: PrivateKey) {
        endCurrentNegotiation()
        let negotiator = PostQuantumKeyNegotiator()

        let gatewayAddress = "10.64.0.1"
        let IPv4Gateway = IPv4Address(gatewayAddress)!
        let endpoint = NWHostEndpoint(hostname: gatewayAddress, port: "\(CONFIG_SERVICE_PORT)")
        let inTunnelTCPConnection = createTCPConnection(endpoint)

        // This will become the new private key of the device
        let ephemeralSharedKey = PrivateKey()

        let tcpConnectionTimeout = keyExchangeRetriesIterator.next() ?? .seconds(10)
        // If the connection never becomes viable, force a reconnection after 10 seconds
        scheduleInTunnelConnectionTimeout(startTime: .now() + tcpConnectionTimeout)

        let tcpConnectionObserver = inTunnelTCPConnection.observe(\.isViable, options: [
            .initial,
            .new,
        ]) { [weak self] observedConnection, _ in
            guard let self, observedConnection.isViable else { return }
            self.negotiation?.tcpConnectionObserver.invalidate()
            self.timer?.cancel()

            if !negotiator.startNegotiation(
                gatewayIP: IPv4Gateway,
                devicePublicKey: privateKey.publicKey,
                presharedKey: ephemeralSharedKey,
                packetTunnel: packetTunnel,
                tcpConnection: inTunnelTCPConnection,
                postQuantumKeyExchangeTimeout: tcpConnectionTimeout
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

    /// Cancels the ongoing key exchange.
    func endCurrentNegotiation() {
        negotiation?.cancel()
        negotiation = nil
    }

    /// Resets the exponential timeout for successful key exchanges, and ends the current key exchange.
    func reset() {
        keyExchangeRetriesIterator = REST.RetryStrategy.postQuantumKeyExchange.makeDelayIterator()
        endCurrentNegotiation()
    }

    private func scheduleInTunnelConnectionTimeout(startTime: DispatchWallTime) {
        let newTimer = DispatchSource.makeTimerSource()

        newTimer.setEventHandler { [weak self] in
            self?.onFailure()
            self?.timer?.cancel()
        }

        newTimer.schedule(wallDeadline: startTime)
        newTimer.activate()

        timer?.cancel()
        timer = newTimer
    }
}
