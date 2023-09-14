//
//  States.swift
//  PacketTunnel
//
//  Created by pronebird on 07/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import struct RelaySelector.RelaySelectorResult
import TunnelObfuscation
import class WireGuardKitTypes.PrivateKey

/**
 Tunnel actor state with metadata describing the current phase of packet tunnel lifecycle.

 ## General lifecycle

 Packet tunnel always begins in `.initial` state and ends `.disconnected` state over time. Packet tunnel process is not recycled and hence once
 the `.disconnected` state is reached, the process is terminated. The new process is started next time VPN is activated.

 ```
 .initial → .connecting → .connected → .disconnecting → .disconnected
 ```

 ## Reconnecting state

 `.reconnecting` can be entered from `.connected` state.

 ```
 .connected → .reconnecting -> .connected
 .reconnecting → .disconnecting → .disconnected
 ```

 ### Packet tunnel considerations

 Packet tunnel should raise `NEPacketTunnelProvider.isReasserting` when `reconnecting` but not when `connecting` since
 `isReasserting = false` always leads to `NEVPNStatus.`

 ## Interruption

 `.connecting` or `.reconnecting` can be interrupted if the tunnel is requested to stop, which should segue actor towards `.disconnected` state.

 */
public enum State {
    /// Initial state at the time when actor is initialized but before the first connection attempt.
    case initial

    /// Tunnel is attempting to connect.
    /// The actor should remain in this state until the very first connection is established, i.e determined by tunnel monitor.
    case connecting(ConnectionState)

    /// Tunnel is connected.
    case connected(ConnectionState)

    /// Tunnel is attempting to reconnect.
    case reconnecting(ConnectionState)

    /// Tunnel is disconnecting.
    case disconnecting(ConnectionState)

    /// Tunnel is disconnected.
    /// Normally the process is shutdown after entering this state.
    case disconnected

    /// Error state.
    /// This state is normally entered when the tunnel is unable to start or reconnect.
    /// In this state the tunnel blocks all nework connectivity by setting up a peerless WireGuard tunnel, and either awaits user action or, in cirtain
    /// circumstances, attempts to recover automatically using a repeating timer.
    case error(BlockedState)
}

/// Policy describing what WG key to use for tunnel communication.
public enum KeyPolicy {
    /// Use current key stored in device data.
    case useCurrent

    /// Use prior key until timer fires.
    case usePrior(_ priorKey: PrivateKey, _ timerTask: AutoCancellingTask)
}

/// Enum describing network availability.
public enum NetworkReachability: Equatable {
    case undetermined, reachable, unreachable
}

public struct ConnectionState {
    /// Current selected relay.
    public var selectedRelay: RelaySelectorResult

    /// Last relay constraints read from settings.
    /// This is primarily used by packet tunnel for updating constraints in tunnel provider.
    public var relayConstraints: RelayConstraints

    /// Last WG key read from setings.
    public var currentKey: PrivateKey

    /// Policy describing the current key that should be used by the tunnel.
    public var keyPolicy: KeyPolicy

    /// Whether network connectivity outside of tunnel is available.
    public var networkReachability: NetworkReachability

    /// Connection attempt counter.
    /// Reset to zero once connection is established.
    public var connectionAttemptCount: UInt

    /// Last time packet tunnel rotated the key.
    public var lastKeyRotation: Date?

    /// Increment connection attempt counter by one, wrapping to zero on overflow.
    public mutating func incrementAttemptCount() {
        let (value, isOverflow) = connectionAttemptCount.addingReportingOverflow(1)
        connectionAttemptCount = isOverflow ? 0 : value
    }

    /// Evaluates `keyPolicy` and returns the active key that should be used with tunnel adapter.
    public var activeKey: PrivateKey {
        switch keyPolicy {
        case .useCurrent:
            return currentKey
        case let .usePrior(priorKey, _):
            return priorKey
        }
    }
}

public struct BlockedState {
    /// Error leading to blocked state.
    public var error: Error

    /// Last relay constraints read from settings.
    /// This is primarily used by packet tunnel for updating constraints in tunnel provider.
    public var relayConstraints: RelayConstraints?

    /// Last WG key read from setings.
    public var currentKey: PrivateKey?

    /// Policy describing the current key that should be used by the tunnel.
    public var keyPolicy: KeyPolicy

    /// Whether network connectivity outside of tunnel is available.
    public var networkReachability: NetworkReachability

    /// Last time packet tunnel rotated or attempted to rotate the key.
    /// This is used by `TunnelManager` to detect when it needs to refresh device state from Keychain.
    public var lastKeyRotation: Date?

    /// Task responsible for periodically calling actor to restart the tunnel.
    /// Initiated based on the error that led to blocked state.
    public var recoveryTask: AutoCancellingTask?

    /// Prior state of the actor before entering blocked state
    public var priorState: StatePriorToBlockedState
}

extension BlockedState {
    public var blockStateReason: BlockStateReason {
        // TODO: match error in a separate dependency
        return .noRelaysSatisfyingConstraints
    }
}

public enum StatePriorToBlockedState {
    case initial, connecting, connected, reconnecting
}

/**
 Target state the actor should transition into upon request to either start (connect) or reconnect.
 */
public enum TargetStateForReconnect {
    case reconnecting, connecting
}

public enum BlockedStateError: LocalizedError {
    case deviceRevoked, invalidAccount

    public var errorDescription: String? {
        switch self {
        case .deviceRevoked:
            return "Device is revoked."
        case .invalidAccount:
            return "Account is invalid."
        }
    }
}

public enum NextRelay {
    /// Select next relay randomly.
    case random

    /// Use currently selected relay, fallback to random if not set.
    case current

    /// Use pre-selected relay.
    case preSelected(RelaySelectorResult)
}
