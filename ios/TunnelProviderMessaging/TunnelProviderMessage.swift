//
//  TunnelProviderMessage.swift
//  TunnelProviderMessaging
//
//  Created by pronebird on 27/07/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import RelaySelector

/// Enum describing supported app messages handled by packet tunnel provider.
public enum TunnelProviderMessage: Codable, CustomStringConvertible {
    /// Request the tunnel to reconnect.
    /// The packet tunnel reconnects to the current relay when selector result is `nil`.
    case reconnectTunnel(RelaySelectorResult?)

    /// Request the tunnel status.
    case getTunnelStatus

    /// Send HTTP request outside of VPN tunnel.
    case sendURLRequest(ProxyURLRequest)

    /// Cancel HTTP request sent outside of VPN tunnel.
    case cancelURLRequest(UUID)

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
        }
    }

    public init(messageData: Data) throws {
        self = try JSONDecoder().decode(Self.self, from: messageData)
    }

    public func encode() throws -> Data {
        return try JSONEncoder().encode(self)
    }
}
