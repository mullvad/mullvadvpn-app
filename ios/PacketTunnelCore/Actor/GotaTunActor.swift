//
//  GotaTunActor.swift
//  PacketTunnel
//
//  Created by Mullvad VPN.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadTypes
import Network

/// Stub actor for GotaTun tunnel implementation.
/// Implements `PacketTunnelActorProtocol` with no-op methods — the real
/// GotaTun logic will be filled in later.
public final class GotaTunActor: PacketTunnelActorProtocol, @unchecked Sendable {
    private let logger = Logger(label: "GotaTunActor")

    public var observedState: ObservedState {
        get async { .disconnected }
    }

    public var observedStates: AsyncStream<ObservedState> {
        get async {
            AsyncStream { continuation in
                continuation.yield(.disconnected)
                continuation.finish()
            }
        }
    }

    public init() {
        logger.info("GotaTunActor initialized (stub)")
    }

    public func start(options: StartOptions) {
        logger.info("start called (no-op)")
    }

    public func stop() {
        logger.info("stop called (no-op)")
    }

    public func waitUntilDisconnected() async {
        logger.info("waitUntilDisconnected called (no-op)")
    }

    public func onSleep() {
        logger.info("onSleep called (no-op)")
    }

    public func onWake() {
        logger.info("onWake called (no-op)")
    }

    public func updateNetworkReachability(networkPathStatus: NWPath.Status) {
        logger.info("updateNetworkReachability called (no-op)")
    }

    public func reconnect(to nextRelays: NextRelays, reconnectReason: ActorReconnectReason) {
        logger.info("reconnect called (no-op)")
    }

    public func notifyKeyRotation(date: Date?) {
        logger.info("notifyKeyRotation called (no-op)")
    }

    public func setErrorState(reason: BlockedStateReason) {
        logger.info("setErrorState called (no-op)")
    }

    public func notifyEphemeralPeerNegotiated() {
        logger.info("notifyEphemeralPeerNegotiated called (no-op)")
    }

    public func changeEphemeralPeerNegotiationState(
        configuration: EphemeralPeerNegotiationState,
        reconfigurationSemaphore: OneshotChannel
    ) {
        logger.info("changeEphemeralPeerNegotiationState called (no-op)")
    }
}
