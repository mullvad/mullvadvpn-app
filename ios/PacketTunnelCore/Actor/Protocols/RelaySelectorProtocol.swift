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
    func selectRelay(with constraints: RelayConstraints, connectionAttemptFailureCount: UInt) throws -> SelectedRelay
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
