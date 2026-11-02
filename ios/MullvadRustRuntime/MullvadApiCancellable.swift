// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadTypes

public final class MullvadApiCancellable: Cancellable, Sendable {
    private let handle: Task<ApiResponse, Error>

    public init(_ task: @Sendable @escaping () async throws -> ApiResponse) {
        self.handle = Task {
            try await task()
        }
    }

    public func start(_ completion: (@Sendable (ApiResponse) -> Void)?) {
        Task {
            let response: ApiResponse
            do {
                response = try await handle.value
            } catch {
                if let error = error as? ApiContextError,
                    case let .ApiError(statusCode, errorDescription) = error
                {
                    response = .error(
                        statusCode: statusCode,
                        errorDescription: nil,
                        serverResponseCode: errorDescription)
                } else {
                    response = .error(
                        statusCode: 0,
                        errorDescription: error.localizedDescription,
                        serverResponseCode: nil)
                }
            }

            completion?(response)
        }
    }

    public func cancel() {
        handle.cancel()
    }
}
