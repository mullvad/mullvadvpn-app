//
//  TunnelProviderMessage.swift
//  MullvadVPN
//
//  Created by pronebird on 27/07/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Enum describing supported app messages handled by packet tunnel provider.
enum TunnelProviderMessage: Codable, CustomStringConvertible {
    /// Request the tunnel to reconnect.
    /// The packet tunnel reconnects to the current relay when selector result is `nil`.
    case reconnectTunnel(RelaySelectorResult?)

    /// Request the tunnel status.
    case getTunnelStatus

    /// Request the tunnel to transport a http request.
    case transportHTTPRequest(Data)

    /// Request the tunnel to cancel transport for http request.
    case cancelURLRequest(UUID)

    var description: String {
        switch self {
        case .reconnectTunnel:
            return "reconnect-tunnel"
        case .getTunnelStatus:
            return "get-tunnel-status"
        case .transportHTTPRequest:
            return "transport-http-request"
        case .cancelURLRequest:
            return "cancel-transport-http-request"
        }
    }

    init(messageData: Data) throws {
        self = try JSONDecoder().decode(Self.self, from: messageData)
    }

    func encode() throws -> Data {
        return try JSONEncoder().encode(self)
    }
}

/// Container type for tunnel provider replies.
/// The primary purpose of this type is to provide a top level object for `JSONEncoder`, since
/// objects and arrays are the only allowed top level objects on iOS 12.
struct TunnelProviderReply<T: Codable>: Codable {
    var value: T

    init(_ value: T) {
        self.value = value
    }

    init(messageData: Data) throws {
        self = try JSONDecoder().decode(Self.self, from: messageData)
    }

    func encode() throws -> Data {
        return try JSONEncoder().encode(self)
    }
}
