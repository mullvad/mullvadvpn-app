//
//  Actor+.swift
//  PacketTunnelCore
//
//  Created by pronebird on 07/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension PacketTunnelActor {
    /// Returns a stream yielding new value when `state` changes.
    /// The stream starts with current `state` and ends upon moving to `.disconnected` state.
    public var states: AsyncStream<State> {
        AsyncStream { continuation in
            let cancellable = self.$state.sink { newState in
                continuation.yield(newState)

                // Finish stream once entered `.disconnected` state.
                if case .disconnected = newState {
                    continuation.finish()
                }
            }

            continuation.onTermination = { _ in
                cancellable.cancel()
            }
        }
    }
}
