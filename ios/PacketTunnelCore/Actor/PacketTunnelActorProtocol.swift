//
//  PacketTunnelActorProtocol.swift
//  PacketTunnelCoreTests
//
//  Created by Jon Petersson on 2023-10-11.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import Network

public protocol PacketTunnelActorProtocol {
    // State observation
    var observedState: ObservedState { get async }
    var observedStates: AsyncStream<ObservedState> { get async }

    // Lifecycle
    func start(options: StartOptions) async
    func stop() async
    func waitUntilDisconnected() async

    // Sleep cycle
    func onSleep() async
    func onWake()

    // Network
    func updateNetworkReachability(networkPathStatus: NWPath.Status) async

    // Reconnection & key rotation
    func reconnect(to nextRelays: NextRelays, reconnectReason: ActorReconnectReason) async
    func notifyKeyRotation(date: Date?) async

    // Error state
    func setErrorState(reason: BlockedStateReason) async

    // Ephemeral peer negotiation
    func notifyEphemeralPeerNegotiated() async
    func changeEphemeralPeerNegotiationState(
        configuration: EphemeralPeerNegotiationState,
        reconfigurationSemaphore: OneshotChannel
    ) async
}
