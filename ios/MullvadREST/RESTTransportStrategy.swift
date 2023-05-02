//
//  RESTTransportStrategy.swift
//  MullvadREST
//
//  Created by Marco Nikic on 2023-04-27.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension REST {
    public struct TransportStrategy: Codable {
        /// The different transports suggested by the strategy
        public enum Transport {
            /// Suggests using a direct connection
            case useURLSession
            /// Suggests connecting via Shadowsocks proxy
            case useShadowSocks
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
        /// If `code` is `.timeout`, the next suggested tranpsort will be the opposite of the currently suggested one.
        /// - Parameter code: The type of error that made the connection fail
        public mutating func didFail(code: URLError.Code? = .unknown) {
            // Whenever a timeout is encountered, suggest the next transport mode
            if code == .timedOut {
                connectionAttempts = connectionAttempts > 0 ? 0 : 1
                return
            }
            connectionAttempts += 1
            // Avoid overflowing by resetting back to 0 every 3rd failure
            connectionAttempts = connectionAttempts % 3 == 0 ? 0 : connectionAttempts
        }

        /// The suggested connection transport
        ///
        /// - Returns: `.useURLSession` for every 3rd failed attempt, `.useShadowSocks` otherwise, except for special
        /// circumstances as described in `didFail(code:)`
        public func connectionTransport() -> Transport {
            let counter = connectionAttempts
            if counter % 3 != 0 {
                return .useShadowSocks
            }
            return .useURLSession
        }
    }
}
