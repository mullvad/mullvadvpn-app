// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadLogging
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

    /// Request in-app log entries from the tunnel.
    case getInAppLogs

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
        case .getInAppLogs:
            return "get-in-app-logs"
        }
    }

    public init(messageData: Data) throws {
        self = try JSONDecoder().decode(Self.self, from: messageData)
    }

    public func encode() throws -> Data {
        try JSONEncoder().encode(self)
    }
}
