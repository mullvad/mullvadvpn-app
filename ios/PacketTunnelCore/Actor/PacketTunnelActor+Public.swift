//
//  PacketTunnelActor+Public.swift
//  PacketTunnelCore
//
//  Created by pronebird on 27/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

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
        commandChannel.send(.start(options))
    }

    /**
     Tell actor to stop the tunnel.
     */
    nonisolated public func stop() {
        commandChannel.send(.stop)
    }

    /**
     Tell actor to reconnect the tunnel.

     - Parameter nextRelay: next relay to connect to.
     */
    public nonisolated func reconnect(to nextRelay: NextRelay) {
        commandChannel.send(.reconnect(nextRelay))
    }

    /**
     Tell actor that key rotation took place.

     - Parameter date: date when last key rotation took place.
     */
    nonisolated public func notifyKeyRotation(date: Date?) {
        commandChannel.send(.notifyKeyRotated(date))
    }

    /**
     Tell actor to enter error state.
     */
    nonisolated public func setErrorState(reason: BlockedStateReason) {
        commandChannel.send(.error(reason))
    }
}
