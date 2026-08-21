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
