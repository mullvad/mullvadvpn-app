//
//  RESTRequestExecutor.swift
//  MullvadREST
//
//  Created by pronebird on 21/08/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

import protocol MullvadTypes.Cancellable

public protocol RESTRequestExecutor<Success> {
    associatedtype Success: Sendable

    /// Execute new network request with `.noRetry` strategy and receive the result in a completion handler on main queue.
    func execute(completionHandler: @escaping @Sendable (Result<Success, Swift.Error>) -> Void) -> Cancellable

    /// Execute new network request and receive the result in a completion handler on main queue.
    func execute(
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping @Sendable (Result<Success, Swift.Error>) -> Void
    ) -> Cancellable

    /// Execute new network request with `.noRetry` strategy and receive the result back via async flow.
    func execute() async throws -> Success

    /// Execute new network request and receive the result back via async flow.
    func execute(retryStrategy: REST.RetryStrategy) async throws -> Success
}
