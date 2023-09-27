//
//  RESTRequestExecutor+Stubs.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-10-03.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
@testable import MullvadREST
@testable import MullvadTypes

struct RESTRequestExecutorStub<Success>: RESTRequestExecutor {
    typealias Success = Success

    var success: (() -> Success)?

    func execute(completionHandler: @escaping (Result<Success, Error>) -> Void) -> Cancellable {
        AnyCancellable()
    }

    func execute(
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping (Result<Success, Error>) -> Void
    ) -> Cancellable {
        AnyCancellable()
    }

    func execute() async throws -> Success {
        try await execute(retryStrategy: .noRetry)
    }

    func execute(retryStrategy: REST.RetryStrategy) async throws -> Success {
        guard let success = success else { throw POSIXError(.EINVAL) }

        return success()
    }
}
