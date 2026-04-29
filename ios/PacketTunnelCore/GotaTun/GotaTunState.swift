//
//  GotaTunState.swift
//  PacketTunnelCore
//
//  Created by Mullvad VPN.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes

/// Internal state of the GotaTun actor.
enum GotaTunState: Equatable {
    case initial
    case connecting(GotaTunConnectionInfo)
    case connected(GotaTunConnectionInfo)
    case reconnecting(GotaTunConnectionInfo)
    case error(GotaTunBlockedInfo)
    case disconnected
}

/// Data associated with connecting/connected/reconnecting states.
struct GotaTunConnectionInfo: Equatable {
    var selectedRelays: SelectedRelays
    var relayConstraints: RelayConstraints
    var networkReachability: NetworkReachability
    var connectionAttemptCount: UInt
    var transportLayer: TransportLayer
    var remotePort: UInt16
    var lastKeyRotation: Date?
    var isPostQuantum: Bool
    var isDaitaEnabled: Bool

    mutating func incrementAttemptCount() {
        let (value, isOverflow) = connectionAttemptCount.addingReportingOverflow(1)
        connectionAttemptCount = isOverflow ? 0 : value
    }

    var observedConnectionState: ObservedConnectionState {
        ObservedConnectionState(
            selectedRelays: selectedRelays,
            relayConstraints: relayConstraints,
            networkReachability: networkReachability,
            connectionAttemptCount: connectionAttemptCount,
            transportLayer: transportLayer,
            remotePort: remotePort,
            lastKeyRotation: lastKeyRotation,
            isPostQuantum: isPostQuantum,
            isDaitaEnabled: isDaitaEnabled
        )
    }
}

/// Data associated with the error state.
struct GotaTunBlockedInfo: Equatable {
    var reason: BlockedStateReason
    var relayConstraints: RelayConstraints?
    var priorState: PriorState
    var networkReachability: NetworkReachability

    enum PriorState: Equatable {
        case initial, connecting, connected, reconnecting
    }

    var observedBlockedState: ObservedBlockedState {
        ObservedBlockedState(reason: reason, relayConstraints: relayConstraints)
    }
}

extension GotaTunState {
    var observedState: ObservedState {
        switch self {
        case .initial:
            return .initial
        case let .connecting(info):
            return .connecting(info.observedConnectionState)
        case let .connected(info):
            return .connected(info.observedConnectionState)
        case let .reconnecting(info):
            return .reconnecting(info.observedConnectionState)
        case let .error(blocked):
            return .error(blocked.observedBlockedState)
        case .disconnected:
            return .disconnected
        }
    }
}

/// Timing configuration for the GotaTun actor.
public struct GotaTunActorTimings: Sendable {
    /// How often the recovery task retries when in a recoverable error state.
    public let bootRecoveryPeriodicity: Duration

    /// How long to wait after a key rotation before switching to the new key.
    public let wgKeyPropagationDelay: Duration

    public init(
        bootRecoveryPeriodicity: Duration = .seconds(5),
        wgKeyPropagationDelay: Duration = .seconds(120)
    ) {
        self.bootRecoveryPeriodicity = bootRecoveryPeriodicity
        self.wgKeyPropagationDelay = wgKeyPropagationDelay
    }
}
