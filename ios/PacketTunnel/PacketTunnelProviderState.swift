//
//  PacketTunnelProviderState.swift
//  PacketTunnel
//
//  Created by pronebird on 03/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension PacketTunnelProvider {
    /// The high level state of tunnel provider.
    ///
    /// Normally tunnel provider lifecycle looks as following:
    ///
    /// .stopped -> .starting -> .started -> .stopping -> .stopped
    ///
    /// It's possible for packet tunnel to transition from starting to stopping state skipping the started state. In that case the completion handler stored in .starting
    /// state must not be called.
    ///
    /// After completing the round trip from stopped to stopped state, the packet tunnel process is terminated. The new one is launched next time.
    ///
    enum State {
        /// Initial and final state of tunnel provider.
        case stopped

        /// Tunnel provider is starting up.
        case starting(completionHandler: (Error?) -> Void)

        /// Tunnel provider has started.
        case started

        /// Tunnel provider is preparing to stop.
        case stopping(completionHandler: () -> Void)
    }

    /// Representation of `State` with stripped associated values.
    enum PrimitiveState: Equatable {
        case stopped, starting, started, stopping
    }
}

extension PacketTunnelProvider.State {
    typealias PrimitiveState = PacketTunnelProvider.PrimitiveState

    /// Returns `PrimitiveState` variant matching current `State`.
    var primitiveState: PrimitiveState {
        switch self {
        case .stopped:
            return .stopped
        case .starting:
            return .starting
        case .started:
            return .started
        case .stopping:
            return .stopping
        }
    }

    /// Returns `true` if transition to the target state is legal.
    func canTransition(to targetState: PrimitiveState) -> Bool {
        switch (primitiveState, targetState) {
        case (.stopped, .starting):
            return true
        case (.starting, .started):
            return true
        case (.started, .stopping):
            return true
        case (.stopping, .stopped):
            return true
        case (.starting, .stopping):
            return true
        default:
            return false
        }
    }
}
