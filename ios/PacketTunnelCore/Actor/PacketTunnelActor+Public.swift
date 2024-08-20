//
//  PacketTunnelActor+Public.swift
//  PacketTunnelCore
//
//  Created by pronebird on 27/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import WireGuardKitTypes

/**
 Public methods for dispatching commands to Actor.

 - All methods in this extension are `nonisolated` because the channel they use to pass commands for execution is `nonisolated` too.
 - FIFO order is guaranteed for all these calls for as long as they are not invoked simultaneously from multiple concurrent queues.
 - There is no way to wait for these tasks to complete, some of them may even be coalesced and never execute. Observe the `state` to follow the progress.
 */
extension PacketTunnelActor {
    /**
     Tell actor to start the tunnel.

     - Parameter options: start options.
     */
    nonisolated public func start(options: StartOptions) {
        eventChannel.send(.start(options))
    }

    /**
     Tell actor to stop the tunnel.
     */
    nonisolated public func stop() {
        eventChannel.send(.stop)
    }

    /**
     Tell actor to reconnect the tunnel.

     - Parameter nextRelays: next relays to connect to.
     */
    public nonisolated func reconnect(to nextRelays: NextRelays, reconnectReason: ActorReconnectReason) {
        eventChannel.send(.reconnect(nextRelays, reason: reconnectReason))
    }

    /**
     Tell actor that key rotation took place.

     - Parameter date: date when last key rotation took place.
     */
    nonisolated public func notifyKeyRotation(date: Date?) {
        eventChannel.send(.notifyKeyRotated(date))
    }

    /**
     Tell actor that post quantum key exchanging took place.
     */

    nonisolated public func notifyPostQuantumKeyExchanged() {
        eventChannel.send(.notifyPostQuantumKeyExchanged)
    }

    /**
     Issue a new preshared key to the Actor.
     - Parameter key: the new key
     */

    nonisolated public func changeEphemeralPeerNegotiationState(configuration: EphemeralPeerNegotiationState) {
        eventChannel.send(.ephemeralPeerNegotiationStateChanged(configuration))
    }

    /**
     Tell actor to enter error state.
     */
    nonisolated public func setErrorState(reason: BlockedStateReason) {
        eventChannel.send(.error(reason))
    }
}
