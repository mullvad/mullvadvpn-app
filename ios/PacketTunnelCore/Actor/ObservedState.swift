//
//  ObservedState.swift
//  PacketTunnelCore
//
//  Created by pronebird on 11/10/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation
import MullvadTypes
import Network

/// A serializable representation of internal state.
public enum ObservedState: Equatable, Codable {
    case initial
    case connecting(ObservedConnectionState)
    case reconnecting(ObservedConnectionState)
    case connected(ObservedConnectionState)
    case disconnecting(ObservedConnectionState)
    case disconnected
    case error(ObservedBlockedState)
}

/// A serializable representation of internal connection state.
public struct ObservedConnectionState: Equatable, Codable {
    public var selectedRelay: SelectedRelay
    public var relayConstraints: RelayConstraints
    public var networkReachability: NetworkReachability
    public var connectionAttemptCount: UInt
    public var transportLayer: TransportLayer
    public var remotePort: UInt16
    public var lastKeyRotation: Date?

    public var isNetworkReachable: Bool {
        networkReachability != .unreachable
    }

    public init(
        selectedRelay: SelectedRelay,
        relayConstraints: RelayConstraints,
        networkReachability: NetworkReachability,
        connectionAttemptCount: UInt,
        transportLayer: TransportLayer,
        remotePort: UInt16,
        lastKeyRotation: Date? = nil
    ) {
        self.selectedRelay = selectedRelay
        self.relayConstraints = relayConstraints
        self.networkReachability = networkReachability
        self.connectionAttemptCount = connectionAttemptCount
        self.transportLayer = transportLayer
        self.remotePort = remotePort
        self.lastKeyRotation = lastKeyRotation
    }
}

/// A serializable representation of internal blocked state.
public struct ObservedBlockedState: Equatable, Codable {
    public var reason: BlockedStateReason
    public var relayConstraints: RelayConstraints?
}

extension State {
    /// Map `State` to `ObservedState`.
    var observedState: ObservedState {
        switch self {
        case .initial:
            return .initial
        case let .connecting(connState):
            return .connecting(connState.observedConnectionState)
        case let .connected(connState):
            return .connected(connState.observedConnectionState)
        case let .reconnecting(connState):
            return .reconnecting(connState.observedConnectionState)
        case let .disconnecting(connState):
            return .disconnecting(connState.observedConnectionState)
        case .disconnected:
            return .disconnected
        case let .error(blockedState):
            return .error(blockedState.observedBlockedState)
        }
    }
}

extension ConnectionState {
    /// Map `ConnectionState` to `ObservedConnectionState`.
    var observedConnectionState: ObservedConnectionState {
        ObservedConnectionState(
            selectedRelay: selectedRelay,
            relayConstraints: relayConstraints,
            networkReachability: networkReachability,
            connectionAttemptCount: connectionAttemptCount,
            transportLayer: transportLayer,
            remotePort: remotePort,
            lastKeyRotation: lastKeyRotation
        )
    }
}

extension BlockedState {
    /// Map `BlockedState` to `ObservedBlockedState`
    var observedBlockedState: ObservedBlockedState {
        return ObservedBlockedState(reason: reason, relayConstraints: relayConstraints)
    }
}
