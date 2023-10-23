//
//  RESTTransportStrategy.swift
//  MullvadREST
//
//  Created by Marco Nikic on 2023-04-27.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

public struct TransportStrategy: Equatable {
    /// The different transports suggested by the strategy
    public enum Transport {
        /// Suggests using a direct connection
        case useURLSession
        /// Suggests connecting via Shadowsocks proxy
        case useShadowsocks
        /// Suggests connecting via socks proxy
        case useSocks5
    }

    /// The internal counter for suggested transports.
    ///
    /// A value of `0` means  a direct transport suggestion, a value of `1` or `2` means a Shadowsocks transport
    /// suggestion.
    ///
    /// `internal` instead of `private` for testing purposes.
    internal var connectionAttempts: Int

    /// Enables recording of failed connection attempts.
    private let userDefaults: UserDefaults

    /// `UserDefaults` key shared by both processes. Used to cache and synchronize connection attempts between them.
    internal static let connectionAttemptsSharedCacheKey = "ConnectionAttemptsSharedCacheKey"

    public init(_ userDefaults: UserDefaults) {
        self.connectionAttempts = userDefaults.integer(forKey: Self.connectionAttemptsSharedCacheKey)
        self.userDefaults = userDefaults
    }

    /// Instructs the strategy that a network connection failed
    ///
    /// Every third failure results in a direct transport suggestion.
    public mutating func didFail() {
        let (partial, isOverflow) = connectionAttempts.addingReportingOverflow(1)
        // (Int.max - 1) is a multiple of 3, go directly to 2 when overflowing
        // to keep the "every third failure" algorithm correct
        connectionAttempts = isOverflow ? 2 : partial
        userDefaults.set(connectionAttempts, forKey: Self.connectionAttemptsSharedCacheKey)
    }

    /// The suggested connection transport
    ///
    /// - Returns: `.useURLSession` for every 3rd failed attempt, `.useShadowsocks` otherwise
    public func connectionTransport() -> Transport {
        connectionAttempts.isMultiple(of: 3) ? .useURLSession : .useShadowsocks
    }

    public static func == (lhs: TransportStrategy, rhs: TransportStrategy) -> Bool {
        lhs.connectionAttempts == rhs.connectionAttempts
    }
}
