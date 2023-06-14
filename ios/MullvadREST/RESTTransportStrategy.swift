//
//  RESTTransportStrategy.swift
//  MullvadREST
//
//  Created by Marco Nikic on 2023-04-27.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

public struct TransportStrategy: Codable, Equatable {
    /// The different transports suggested by the strategy
    public enum Transport {
        /// Suggests using a direct connection
        case useURLSession
        /// Suggests connecting via Shadowsocks proxy
        case useShadowsocks
    }

    /// The internal counter for suggested transports.
    ///
    /// A value of `0` means  a direct transport suggestion, a value of `1` or `2` means a Shadowsocks transport
    /// suggestion.
    private var connectionAttempts: UInt

    public init() {
        connectionAttempts = 0
    }

    /// Instructs the strategy that a network connection failed
    ///
    /// Every third failure results in a direct transport suggestion.
    public mutating func didFail() {
        connectionAttempts += 1
        // Avoid overflowing by resetting back to 0 every 3rd failure
        connectionAttempts = connectionAttempts.isMultiple(of: 3) ? 0 : connectionAttempts
    }

    /// The suggested connection transport
    ///
    /// - Returns: `.useURLSession` for every 3rd failed attempt, `.useShadowsocks` otherwise
    public func connectionTransport() -> Transport {
        connectionAttempts.isMultiple(of: 3) ? .useURLSession : .useShadowsocks
    }
}
