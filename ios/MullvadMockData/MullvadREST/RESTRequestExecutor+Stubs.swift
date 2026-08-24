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
import MullvadREST
import MullvadTypes

struct RESTRequestExecutorStub<Success: Sendable>: RESTRequestExecutor {
    var success: (() -> Success)?

    func execute(completionHandler: @escaping (Result<Success, Error>) -> Void) -> Cancellable {
        if let result = success?() {
            completionHandler(.success(result))
        }
        return AnyCancellable()
    }

    func execute(
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping (Result<Success, Error>) -> Void
    ) -> Cancellable {
        if let result = success?() {
            completionHandler(.success(result))
        }
        return AnyCancellable()
    }

    func execute() async throws -> Success {
        try await execute(retryStrategy: .noRetry)
    }

    func execute(retryStrategy: REST.RetryStrategy) async throws -> Success {
        guard let success = success else { throw POSIXError(.EINVAL) }

        return success()
    }
}
