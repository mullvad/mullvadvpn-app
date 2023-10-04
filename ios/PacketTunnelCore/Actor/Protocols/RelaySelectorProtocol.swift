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
    public var endpoint: MullvadEndpoint

    /// Relay hostname.
    public var hostname: String

    /// Relay geo location.
    public var location: Location

    /// Designated initializer.
    public init(endpoint: MullvadEndpoint, hostname: String, location: Location) {
        self.endpoint = endpoint
        self.hostname = hostname
        self.location = location
    }
}

extension SelectedRelay {
    /// Converts `SelectedRelay` to `PacketTunnelRelay` for sharing with UI.
    public var packetTunnelRelay: PacketTunnelRelay {
        PacketTunnelRelay(
            ipv4Relay: endpoint.ipv4Relay,
            ipv6Relay: endpoint.ipv6Relay,
            hostname: hostname,
            location: location
        )
    }
}
