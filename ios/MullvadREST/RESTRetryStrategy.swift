//
//  RESTRetryStrategy.swift
//  MullvadREST
//
//  Created by pronebird on 09/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension REST {
    public struct RetryStrategy {
        public var maxRetryCount: Int
        public var retryDelay: DispatchTimeInterval

        /// Strategy configured to never retry.
        public static var noRetry = RetryStrategy(maxRetryCount: 0, retryDelay: .never)

        /// Startegy configured with 3 retry attempts with 2 seconds delay between.
        public static var `default` = RetryStrategy(maxRetryCount: 3, retryDelay: .seconds(2))

        /// Strategy configured with 10 retry attempts with 2 seconds delay between.
        public static var aggressive = RetryStrategy(maxRetryCount: 10, retryDelay: .seconds(2))
    }
}
