//
//  TunnelProviderMessage.swift
//  PacketTunnelCore
//
//  Created by pronebird on 27/07/2021.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST

/// Enum describing supported app messages handled by packet tunnel provider.
public enum TunnelProviderMessage: Codable, CustomStringConvertible {
    /// Request the tunnel to reconnect.
    case reconnectTunnel(NextRelays)

    /// Request the tunnel status.
    case getTunnelStatus

    /// Send API request outside of VPN tunnel.
    case sendAPIRequest(ProxyAPIRequest)

    /// Cancel API request sent outside of VPN tunnel.
    case cancelAPIRequest(UUID)

    /// Notify tunnel about private key rotation.
    case privateKeyRotation

    public var description: String {
        switch self {
        case .reconnectTunnel:
            return "reconnect-tunnel"
        case .getTunnelStatus:
            return "get-tunnel-status"
        case .sendAPIRequest:
            return "send-api-request"
        case .cancelAPIRequest:
            return "cancel-api-request"
        case .privateKeyRotation:
            return "private-key-rotation"
        }
    }

    public init(messageData: Data) throws {
        self = try JSONDecoder().decode(Self.self, from: messageData)
    }

    public func encode() throws -> Data {
        try JSONEncoder().encode(self)
    }
}
