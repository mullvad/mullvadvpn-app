//
//  TransportStrategy.swift
//  MullvadREST
//
//  Created by Marco Nikic on 2023-04-27.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

public struct TransportStrategy: Equatable {
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
    ///
    /// `internal` instead of `private` for testing purposes.
    internal var connectionAttempts: Int

    /// Enables recording of failed connection attempts.
    private let attemptsRecorder: AttemptsRecording

    public init(connectionAttempts: Int = 0, attemptsRecorder: AttemptsRecording) {
        self.connectionAttempts = connectionAttempts
        self.attemptsRecorder = attemptsRecorder
    }

    /// Instructs the strategy that a network connection failed
    ///
    /// Every third failure results in a direct transport suggestion.
    public mutating func didFail() {
        let (partial, isOverflow) = connectionAttempts.addingReportingOverflow(1)
        // (Int.max - 1) is a multiple of 3, go directly to 2 when overflowing
        connectionAttempts = isOverflow ? 2 : partial
        attemptsRecorder.record(connectionAttempts)
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
