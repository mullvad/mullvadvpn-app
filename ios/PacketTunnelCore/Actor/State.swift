//
//  States.swift
//  PacketTunnel
//
//  Created by pronebird on 07/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
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
public enum UsedKeyPolicy {
    /// Use current key stored in device data.
    case useCurrent

    /// Use prior key until timer fires.
    case usePrior(_ priorKey: PrivateKey, _ timerTask: AutoCancellingTask)
}

/// Enum describing network availability.
public enum NetworkReachability {
    case undetermined, reachable, unreachable
}

public struct ConnectionState {
    /// Current selected relay.
    public var selectedRelay: RelaySelectorResult

    /// Policy describing the current key that should be used by the tunnel.
    public var keyPolicy: UsedKeyPolicy

    /// Whether network connectivity outside of tunnel is available.
    public var networkReachability: NetworkReachability

    /// Connection attempt counter.
    /// Reset to zero once connection is established.
    public var connectionAttemptCount: UInt

    /// Increment connection attempt counter by one, wrapping to zero on overflow.
    public mutating func incrementAttemptCount() {
        let (value, isOverflow) = connectionAttemptCount.addingReportingOverflow(1)
        connectionAttemptCount = isOverflow ? 0 : value
    }
}

public struct BlockedState {
    /// Error leading to blocked state.
    public var error: Error

    /// Policy describing the current key that should be used by the tunnel.
    public var keyPolicy: UsedKeyPolicy

    /// Task responsible for periodically calling actor to restart the tunnel.
    /// Initiated based on the error that led to blocked state.
    public var recoveryTask: AutoCancellingTask?

    /// Prior state of the actor before entering blocked state
    public var priorState: StatePriorToBlockedState
}

public enum ReconnectionSource {
    case connectionState(ConnectionState)
    case blockedState(BlockedState)
}

public enum StatePriorToBlockedState {
    case initial, connecting, connected, reconnecting
}

public struct DeviceRevokedError: LocalizedError {
    public var errorDescription: String? {
        return "Device is revoked."
    }
}

extension State {
    public var priorState: StatePriorToBlockedState? {
        switch self {
        case .initial:
            return .initial
        case .connecting:
            return .connecting
        case .connected:
            return .connected
        case .reconnecting:
            return .reconnecting
        case .disconnecting, .disconnected, .error:
            return nil
        }
    }
}
