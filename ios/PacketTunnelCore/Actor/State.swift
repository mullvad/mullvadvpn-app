//
//  State.swift
//  PacketTunnel
//
//  Created by pronebird on 07/08/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadRustRuntime
import MullvadSettings
import MullvadTypes
@preconcurrency import WireGuardKitTypes

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

 ## Error state

 `.error` can be entered from nearly any other state except when the tunnel is at or past `.disconnecting` phase.

 A call to reconnect the tunnel while in error state can be used to attempt the recovery and exit the error state upon success.
 Note that actor decides the target state when transitioning from `.error` state forward based on state prior to error state.

 ```
 .error → .reconnecting
 .error → .connecting
 ```

 ### Packet tunnel considerations

 Packet tunnel should raise `NEPacketTunnelProvider.reasserting` when `reconnecting` but not when `connecting` since
 `reasserting = false` always leads to `NEVPNStatus.connected`.

 ## Interruption

 `.connecting`, `.reconnecting`, `.error` can be interrupted if the tunnel is requested to stop, which should segue actor towards `.disconnected` state.

 */
enum State: Equatable {
    /// Initial state at the time when actor is initialized but before the first connection attempt.
    case initial

    /// Establish a connection to the gateway, and exchange an ephemeral wireguard peer with the GRPC service that resides there.
    case negotiatingEphemeralPeer(ConnectionData, PrivateKey)

    /// Tunnel is attempting to connect.
    /// The actor should remain in this state until the very first connection is established, i.e determined by tunnel monitor.
    case connecting(ConnectionData)

    /// Tunnel is connected.
    case connected(ConnectionData)

    /// Tunnel is attempting to reconnect.
    case reconnecting(ConnectionData)

    /// Tunnel is disconnecting.
    case disconnecting(ConnectionData)

    /// Tunnel is disconnected.
    /// Normally the process is shutdown after entering this state.
    case disconnected

    /// Error state.
    /// This state is normally entered when the tunnel is unable to start or reconnect.
    /// In this state the tunnel blocks all nework connectivity by setting up a peerless WireGuard tunnel, and either awaits user action or, in certain
    /// circumstances, attempts to recover automatically using a repeating timer.
    case error(BlockingData)
}

/// Enum describing network availability.
public enum NetworkReachability: Equatable, Codable, Sendable {
    case undetermined, reachable, unreachable
}

protocol StateAssociatedData {
    var currentKey: PrivateKey? { get set }
    var keyPolicy: State.KeyPolicy { get set }
    var networkReachability: NetworkReachability { get set }
    var lastKeyRotation: Date? { get set }
}

extension State {
    /// Policy describing what WG key to use for tunnel communication.
    enum KeyPolicy: Sendable {
        /// Use current key stored in device data.
        case useCurrent

        /// Use prior key until timer fires.
        case usePrior(_ priorKey: PrivateKey, _ timerTask: AutoCancellingTask)
    }

    /// Data associated with states that hold connection data.
    struct ConnectionData: Equatable, StateAssociatedData {
        /// Current selected relays.
        public var selectedRelays: SelectedRelays

        /// Last relay constraints read from settings.
        /// This is primarily used by packet tunnel for updating constraints in tunnel provider.
        public var relayConstraints: RelayConstraints

        /// Last WG key read from settings.
        /// Can be `nil` if moved to `keyPolicy`.
        public var currentKey: PrivateKey?

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

        /// The actual endpoint fed to WireGuard, can be a local endpoint if obfuscation is used.
        public var connectedEndpoint: MullvadEndpoint
        /// Via which transport protocol was the connection made to the relay
        public let transportLayer: TransportLayer

        /// The remote port that was chosen to connect to `connectedEndpoint`
        public var remotePort: UInt16

        /// True if post-quantum key exchange is enabled
        public let isPostQuantum: Bool

        /// True if Daita is enabled
        public let isDaitaEnabled: Bool

        /// The obfuscation method in force on the connection
        public let obfuscationMethod: WireGuardObfuscationState
    }

    /// Data associated with error state.
    struct BlockingData: StateAssociatedData, Sendable {
        /// Reason why block state was entered.
        public var reason: BlockedStateReason

        /// Last relay constraints read from settings.
        /// This is primarily used by packet tunnel for updating constraints in tunnel provider.
        public var relayConstraints: RelayConstraints?

        /// Last WG key read from setings.
        /// Can be `nil` if moved to `keyPolicy` or when it's uknown.
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
        public var priorState: PriorState
    }
}

/// Reason why packet tunnel entered error state.
public enum BlockedStateReason: String, Codable, Equatable, Sendable {
    /// Device is locked.
    case deviceLocked

    /// Settings schema is outdated.
    case outdatedSchema

    /// General error for no relays satisfying constraints.
    case noRelaysSatisfyingConstraints

    /// No relays satisfying filter constraints.
    case noRelaysSatisfyingFilterConstraints

    /// No relays satisfying multihop constraints.
    case multihopEntryEqualsExit

    /// No relays satisfying DAITA constraints.
    case noRelaysSatisfyingDaitaConstraints

    /// No relays satisfying DAITA constraints.
    case noRelaysSatisfyingObfuscationSettings

    /// Any other failure when reading settings.
    case readSettings

    /// Invalid account.
    case invalidAccount

    /// Account is expired.
    case accountExpired

    /// Device revoked.
    case deviceRevoked

    /// Device is logged out.
    /// This is an extreme edge case, most likely means that main bundle forgot to delete the VPN configuration during logout.
    case deviceLoggedOut

    /// Tunnel adapter error.
    case tunnelAdapter

    /// Invalid public key.
    case invalidRelayPublicKey

    /// Unidentified reason.
    case unknown
}

extension State.BlockingData {
    /// Legal states that can precede error state.
    enum PriorState: Equatable {
        case initial, connecting, connected, reconnecting
    }
}

/// Describes which relay the tunnel should connect to next.
public enum NextRelays: Equatable, Codable, Sendable {
    /// Select next relays randomly.
    case random

    /// Use currently selected relays, fallback to random if not set.
    case current

    /// Use pre-selected relays.
    case preSelected(SelectedRelays)
}

/// Describes the reason for reconnection request.
public enum ActorReconnectReason: Equatable, Sendable {
    /// Initiated by user.
    case userInitiated

    /// Initiated by tunnel monitor due to loss of connectivity, or if ephemeral peer negotiation times out.
    /// Actor will increment the connection attempt counter before picking next relay.
    case connectionLoss
}
