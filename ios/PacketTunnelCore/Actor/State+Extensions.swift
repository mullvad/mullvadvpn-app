//
//  State+Extensions.swift
//  PacketTunnelCore
//
//  Created by pronebird on 08/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import WireGuardKitTypes

extension State {
    /// Returns the target state to which the actor state should transition when requested to reconnect.
    /// It returns `nil` when reconnection is not supported such as when already `.disconnecting` or `.disconnected` states.
    var targetStateForReconnect: TargetStateForReconnect? {
        switch self {
        case .initial:
            return .connecting

        case .connecting:
            return .connecting

        case .connected, .reconnecting:
            return .reconnecting

        case let .error(blockedState):
            switch blockedState.priorState {
            case .initial, .connecting:
                return .connecting
            case .connected, .reconnecting:
                return .reconnecting
            }

        case .disconnecting, .disconnected:
            return nil
        }
    }

    // MARK: - Logging

    func logFormat() -> String {
        switch self {
        case let .connecting(connState), let .connected(connState), let .reconnecting(connState):
            let hostname = connState.selectedRelay.hostname

            return """
            \(name) to \(hostname), \
            key: \(connState.keyPolicy.logFormat()), \
            net: \(connState.networkReachability), \
            attempt: \(connState.connectionAttemptCount)
            """

        case let .error(blockedState):
            return "\(name): \(blockedState.reason)"

        case .initial, .disconnecting, .disconnected:
            return name
        }
    }

    var name: String {
        switch self {
        case .connected:
            return "Connected"
        case .connecting:
            return "Connecting"
        case .reconnecting:
            return "Reconnecting"
        case .disconnecting:
            return "Disconnecting"
        case .disconnected:
            return "Disconnected"
        case .initial:
            return "Initial"
        case .error:
            return "Error"
        }
    }
}

extension KeyPolicy {
    func logFormat() -> String {
        switch self {
        case .useCurrent:
            return "current"
        case .usePrior:
            return "prior"
        }
    }
}

extension BlockedStateReason {
    /**
     Returns true if the tunnel should attempt to restart periodically to recover from error that does not require explicit restart to be initiated by user.

     Common scenarios when tunnel will attempt to restart itself periodically:

     - Keychain and filesystem are locked on boot until user unlocks device in the very first time.
     - App update that requires settings schema migration. Packet tunnel will be automatically restarted after update but it would not be able to read settings until
       user opens the app which performs migration.
     */
    var shouldRestartAutomatically: Bool {
        switch self {
        case .deviceLocked:
            return true

        case .noRelaysSatisfyingConstraints, .readSettings, .invalidAccount, .accountExpired, .deviceRevoked,
             .tunnelAdapter, .unknown, .deviceLoggedOut, .outdatedSchema, .invalidRelayPublicKey:
            return false
        }
    }
}
