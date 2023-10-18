//
//  Actor+.swift
//  PacketTunnelCore
//
//  Created by pronebird on 07/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension PacketTunnelActor {
    /// Returns a stream yielding `ObservedState`.
    /// Note that the stream yields current value when created.
    public var observedStates: AsyncStream<ObservedState> {
        AsyncStream { continuation in
            let cancellable = $observedState.sink { newState in
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

    /// Wait until the `observedState` moved to `.connected`.
    /// Should return if the state is `.disconnected` as this is the final state of actor.
    public func waitUntilConnected() async {
        for await newState in observedStates {
            switch newState {
            case .connected, .disconnected:
                // Return once either desired or final state is reached.
                return

            case .connecting, .disconnecting, .error, .initial, .reconnecting:
                break
            }
        }
    }

    /// Wait until the `observedState` moved to `.disiconnected`.
    public func waitUntilDisconnected() async {
        for await newState in observedStates {
            if case .disconnected = newState {
                return
            }
        }
    }
}
