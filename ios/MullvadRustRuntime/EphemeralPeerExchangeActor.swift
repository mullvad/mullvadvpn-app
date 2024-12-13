//
//  EphemeralPeerExchangeActor.swift
//  PacketTunnel
//
//  Created by Marco Nikic on 2024-04-12.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadRustRuntimeProxy
import MullvadTypes
import NetworkExtension
import WireGuardKitTypes

public protocol EphemeralPeerExchangeActorProtocol {
    func startNegotiation(with privateKey: PrivateKey, enablePostQuantum: Bool, enableDaita: Bool)
    func endCurrentNegotiation()
    func reset()
}

public class EphemeralPeerExchangeActor: EphemeralPeerExchangeActorProtocol {
    struct Negotiation {
        var negotiator: EphemeralPeerNegotiating

        func cancel() {
            negotiator.cancelKeyNegotiation()
        }
    }

    unowned let packetTunnel: any TunnelProvider
    internal var negotiation: Negotiation?
    private var timer: DispatchSourceTimer?
    private var keyExchangeRetriesIterator: AnyIterator<Duration>!
    private let iteratorProvider: () -> AnyIterator<Duration>
    private let negotiationProvider: EphemeralPeerNegotiating.Type

    // Callback in the event of the negotiation failing on startup
    var onFailure: () -> Void

    public init(
        packetTunnel: any TunnelProvider,
        onFailure: @escaping (() -> Void),
        negotiationProvider: EphemeralPeerNegotiating.Type = EphemeralPeerNegotiator.self,
        iteratorProvider: @escaping () -> AnyIterator<Duration>
    ) {
        self.packetTunnel = packetTunnel
        self.onFailure = onFailure
        self.negotiationProvider = negotiationProvider
        self.iteratorProvider = iteratorProvider
        self.keyExchangeRetriesIterator = iteratorProvider()
    }

    /// Starts a new key exchange.
    ///
    /// Any ongoing key negotiation is stopped before starting a new one.
    /// An exponential backoff timer is used to stop the exchange if it takes too long,
    /// or if the TCP connection takes too long to become ready.
    /// It is reset after every successful key exchange.
    ///
    /// - Parameter privateKey: The device's current private key
    public func startNegotiation(with privateKey: PrivateKey, enablePostQuantum: Bool, enableDaita: Bool) {
        endCurrentNegotiation()
        let negotiator = negotiationProvider.init()

        // This will become the new private key of the device
        let ephemeralSharedKey = PrivateKey()

        let tcpConnectionTimeout = keyExchangeRetriesIterator.next() ?? .seconds(10)
        // If the connection never becomes viable, force a reconnection after 10 seconds
        let peerParameters = EphemeralPeerParameters(
            peer_exchange_timeout: UInt64(tcpConnectionTimeout.timeInterval),
            enable_post_quantum: enablePostQuantum,
            enable_daita: enableDaita,
            funcs: mapWgFunctions(functions: packetTunnel.wgFunctions())
        )

        if !negotiator.startNegotiation(
            devicePublicKey: privateKey.publicKey,
            presharedKey: ephemeralSharedKey,
            peerReceiver: packetTunnel,
            ephemeralPeerParams: peerParameters
        ) {
            // Cancel the negotiation to shut down any remaining use of the TCP connection on the Rust side
            self.negotiation?.cancel()
            self.negotiation = nil
            self.onFailure()
        }

        negotiation = Negotiation(
            negotiator: negotiator
        )
    }

    private func mapWgFunctions(functions: WgFunctionPointers) -> WgTcpConnectionFunctions {
        var mappedFunctions = WgTcpConnectionFunctions()

        mappedFunctions.close_fn = functions.close
        mappedFunctions.open_fn = functions.open
        mappedFunctions.send_fn = functions.send
        mappedFunctions.recv_fn = functions.receive

        return mappedFunctions
    }

    /// Cancels the ongoing key exchange.
    public func endCurrentNegotiation() {
        negotiation?.cancel()
        negotiation = nil
    }

    /// Resets the exponential timeout for successful key exchanges, and ends the current key exchange.
    public func reset() {
        keyExchangeRetriesIterator = iteratorProvider()
        endCurrentNegotiation()
    }
}
