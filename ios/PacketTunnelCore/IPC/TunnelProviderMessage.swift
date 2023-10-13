//
//  TunnelProviderMessage.swift
//  PacketTunnelCore
//
//  Created by pronebird on 27/07/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Enum describing supported app messages handled by packet tunnel provider.
public enum TunnelProviderMessage: Codable, CustomStringConvertible {
    /// Request the tunnel to reconnect.
    case reconnectTunnel(NextRelay)

    /// Request the tunnel status.
    case getTunnelStatus

    /// Send HTTP request outside of VPN tunnel.
    case sendURLRequest(ProxyURLRequest)

    /// Cancel HTTP request sent outside of VPN tunnel.
    case cancelURLRequest(UUID)

    /// Notify tunnel about private key rotation.
    case privateKeyRotation

    public var description: String {
        switch self {
        case .reconnectTunnel:
            return "reconnect-tunnel"
        case .getTunnelStatus:
            return "get-tunnel-status"
        case .sendURLRequest:
            return "send-http-request"
        case .cancelURLRequest:
            return "cancel-http-request"
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
