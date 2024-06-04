//
//  RelaySelectorProtocol.swift
//  PacketTunnel
//
//  Created by pronebird on 08/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

/// Protocol describing a type that can select a relay.
public protocol RelaySelectorProtocol {
    func selectRelays(with constraints: RelayConstraints, connectionAttemptCount: UInt) throws -> SelectedRelays
}

/// Struct describing the selected relay.
public struct SelectedRelay: Equatable, Codable {
    /// Selected relay endpoint.
    public let endpoint: MullvadEndpoint

    /// Relay hostname.
    public let hostname: String

    /// Relay geo location.
    public let location: Location

    /// Number of retried attempts to connect to a relay.
    public let retryAttempts: UInt

    /// Designated initializer.
    public init(endpoint: MullvadEndpoint, hostname: String, location: Location, retryAttempts: UInt) {
        self.endpoint = endpoint
        self.hostname = hostname
        self.location = location
        self.retryAttempts = retryAttempts
    }
}

extension SelectedRelay: CustomDebugStringConvertible {
    public var debugDescription: String {
        "\(hostname) -> \(endpoint.ipv4Relay.description)"
    }
}

public struct SelectedRelays: Equatable, Codable {
    public let entry: SelectedRelay?
    public let exit: SelectedRelay

    public init(entry: SelectedRelay?, exit: SelectedRelay) {
        self.entry = entry
        self.exit = exit
    }
}

extension SelectedRelays: CustomDebugStringConvertible {
    public var debugDescription: String {
        "Entry: \(entry?.hostname ?? "-") -> \(entry?.endpoint.ipv4Relay.description ?? "-"), " +
            "Exit: \(exit.hostname) -> \(exit.endpoint.ipv4Relay.description)"
    }
}
