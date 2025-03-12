//
//  RelaySelectorProtocol.swift
//  PacketTunnel
//
//  Created by pronebird on 08/08/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import MullvadTypes

/// Protocol describing a type that can select a relay.
public protocol RelaySelectorProtocol {
    var relayCache: RelayCacheProtocol { get }
    func selectRelays(
        tunnelSettings: LatestTunnelSettings,
        connectionAttemptCount: UInt
    ) throws -> SelectedRelays

    func findCandidates(
        tunnelSettings: LatestTunnelSettings
    ) throws -> RelayCandidates
}

/// Struct describing the selected relay.
public struct SelectedRelay: Equatable, Codable, Sendable {
    /// Selected relay endpoint.
    public let endpoint: MullvadEndpoint

    /// Relay hostname.
    public let hostname: String

    /// Relay geo location.
    public let location: Location

    /// Designated initializer.
    public init(endpoint: MullvadEndpoint, hostname: String, location: Location) {
        self.endpoint = endpoint
        self.hostname = hostname
        self.location = location
    }
}

extension SelectedRelay: CustomDebugStringConvertible {
    public var debugDescription: String {
        "\(hostname) -> \(endpoint.ipv4Relay.description)"
    }
}

public struct SelectedRelays: Equatable, Codable, Sendable {
    public let entry: SelectedRelay?
    public let exit: SelectedRelay
    public let retryAttempt: UInt

    public init(entry: SelectedRelay?, exit: SelectedRelay, retryAttempt: UInt) {
        self.entry = entry
        self.exit = exit
        self.retryAttempt = retryAttempt
    }
}

extension SelectedRelays: CustomDebugStringConvertible {
    public var debugDescription: String {
        "Entry: \(entry?.hostname ?? "-") -> \(entry?.endpoint.ipv4Relay.description ?? "-"), " +
            "Exit: \(exit.hostname) -> \(exit.endpoint.ipv4Relay.description)"
    }
}
