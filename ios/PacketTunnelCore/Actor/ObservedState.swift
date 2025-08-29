//
//  ObservedState.swift
//  PacketTunnelCore
//
//  Created by pronebird on 11/10/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation
import MullvadREST
import MullvadSettings
import MullvadTypes
import Network
@preconcurrency import WireGuardKitTypes

/// A serializable representation of internal state.
public enum ObservedState: Equatable, Codable, Sendable {
    case initial
    case connecting(ObservedConnectionState)
    case reconnecting(ObservedConnectionState)
    case negotiatingEphemeralPeer(ObservedConnectionState, PrivateKey)
    case connected(ObservedConnectionState)
    case disconnecting(ObservedConnectionState)
    case disconnected
    case error(ObservedBlockedState)
}

/// A serializable representation of internal connection state.
public struct ObservedConnectionState: Equatable, Codable, Sendable {
    public var selectedRelays: SelectedRelays
    public var relayConstraints: RelayConstraints
    public var networkReachability: NetworkReachability
    public var connectionAttemptCount: UInt
    public var transportLayer: TransportLayer
    public var remotePort: UInt16
    public var lastKeyRotation: Date?
    public let isPostQuantum: Bool
    public let isDaitaEnabled: Bool
    public let obfuscationMethod: WireGuardObfuscationState

    public var isNetworkReachable: Bool {
        networkReachability != .unreachable
    }

    public init(
        selectedRelays: SelectedRelays,
        relayConstraints: RelayConstraints,
        networkReachability: NetworkReachability,
        connectionAttemptCount: UInt,
        transportLayer: TransportLayer,
        remotePort: UInt16,
        lastKeyRotation: Date? = nil,
        isPostQuantum: Bool,
        isDaitaEnabled: Bool,
        obfuscationMethod: WireGuardObfuscationState = .off
    ) {
        self.selectedRelays = selectedRelays
        self.relayConstraints = relayConstraints
        self.networkReachability = networkReachability
        self.connectionAttemptCount = connectionAttemptCount
        self.transportLayer = transportLayer
        self.remotePort = remotePort
        self.lastKeyRotation = lastKeyRotation
        self.isPostQuantum = isPostQuantum
        self.isDaitaEnabled = isDaitaEnabled
        self.obfuscationMethod = obfuscationMethod
    }
}

/// A serializable representation of internal blocked state.
public struct ObservedBlockedState: Equatable, Codable, Sendable {
    public var reason: BlockedStateReason
    public var relayConstraints: RelayConstraints?

    public init(reason: BlockedStateReason, relayConstraints: RelayConstraints? = nil) {
        self.reason = reason
        self.relayConstraints = relayConstraints
    }
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
        case let .negotiatingEphemeralPeer(connState, privateKey):
            return .negotiatingEphemeralPeer(connState.observedConnectionState, privateKey)
        case .disconnected:
            return .disconnected
        case let .error(blockedState):
            return .error(blockedState.observedBlockedState)
        }
    }
}

extension State.ConnectionData {
    /// Map `State.ConnectionData` to `ObservedConnectionState`.
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
            isDaitaEnabled: isDaitaEnabled,
            obfuscationMethod: obfuscationMethod
        )
    }
}

extension State.BlockingData {
    /// Map `State.BlockingData` to `ObservedBlockedState`
    var observedBlockedState: ObservedBlockedState {
        return ObservedBlockedState(reason: reason, relayConstraints: relayConstraints)
    }
}
