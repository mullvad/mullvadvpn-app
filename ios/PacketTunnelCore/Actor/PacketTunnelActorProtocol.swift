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
    func start(options: StartOptions)
    func stop()
    func waitUntilDisconnected() async

    // Sleep cycle
    func onSleep()
    func onWake()

    // Network
    func updateNetworkReachability(networkPathStatus: NWPath.Status)

    // Reconnection & key rotation
    func reconnect(to nextRelays: NextRelays, reconnectReason: ActorReconnectReason)
    func notifyKeyRotation(date: Date?)

    // Error state
    func setErrorState(reason: BlockedStateReason)

    // Ephemeral peer negotiation
    func notifyEphemeralPeerNegotiated()
    func changeEphemeralPeerNegotiationState(
        configuration: EphemeralPeerNegotiationState,
        reconfigurationSemaphore: OneshotChannel
    )
}
