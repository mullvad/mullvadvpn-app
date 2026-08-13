//
//  Actor+.swift
//  PacketTunnelCore
//
//  Created by pronebird on 07/09/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension PacketTunnelActor {
    /// Returns a stream yielding `ObservedState`.
    /// Note that the stream yields current value when created.
    public var observedStates: AsyncStream<ObservedState> {
        stateBroadcaster.makeStream(replaying: observedState)
    }

    /// Wait until the `observedState` moved to `.disconnected`.
    public func waitUntilDisconnected() async {
        await observedStates.waitUntilDisconnected()
    }
}
