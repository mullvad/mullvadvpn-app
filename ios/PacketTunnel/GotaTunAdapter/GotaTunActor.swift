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
import PacketTunnelCore

/// Stub actor for GotaTun tunnel implementation.
/// Implements `PacketTunnelActorProtocol` with no-op methods — the real
/// GotaTun logic will be filled in later.
final class GotaTunActor: PacketTunnelActorProtocol, @unchecked Sendable {
    private let logger = Logger(label: "GotaTunActor")

    var observedState: ObservedState {
        get async { .disconnected }
    }

    var observedStates: AsyncStream<ObservedState> {
        get async {
            AsyncStream { continuation in
                continuation.yield(.disconnected)
                continuation.finish()
            }
        }
    }

    init() {
        logger.info("GotaTunActor initialized (stub)")
    }

    func start(options: StartOptions) {
        logger.info("start called (no-op)")
    }

    func stop() {
        logger.info("stop called (no-op)")
    }

    func waitUntilDisconnected() async {
        logger.info("waitUntilDisconnected called (no-op)")
    }

    func onSleep() {
        logger.info("onSleep called (no-op)")
    }

    func onWake() {
        logger.info("onWake called (no-op)")
    }

    func updateNetworkReachability(networkPathStatus: NWPath.Status) {
        logger.info("updateNetworkReachability called (no-op)")
    }

    func reconnect(to nextRelays: NextRelays, reconnectReason: ActorReconnectReason) {
        logger.info("reconnect called (no-op)")
    }

    func notifyKeyRotation(date: Date?) {
        logger.info("notifyKeyRotation called (no-op)")
    }

    func setErrorState(reason: BlockedStateReason) {
        logger.info("setErrorState called (no-op)")
    }

    func notifyEphemeralPeerNegotiated() {
        logger.info("notifyEphemeralPeerNegotiated called (no-op)")
    }

    func changeEphemeralPeerNegotiationState(
        configuration: EphemeralPeerNegotiationState,
        reconfigurationSemaphore: OneshotChannel
    ) {
        logger.info("changeEphemeralPeerNegotiationState called (no-op)")
    }
}
