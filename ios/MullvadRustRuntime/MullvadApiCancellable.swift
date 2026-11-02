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
    private enum Handle {
        case legacy(RequestCancelHandle)
        case task(Task<ApiResponse, Error>)
    }

    private let handle: Handle

    public init(handle: consuming RequestCancelHandle) {
        self.handle = .legacy(handle)
    }

    public init(task: @Sendable @escaping () async throws -> ApiResponse) {
        self.handle = .task(
            Task {
                try await task()
            }
        )
    }

    public func start(_ completion: (@Sendable (ApiResponse) throws -> Void)?) {
        let completionCookie = MullvadApiCompletion { apiResponse in
            try? completion?(apiResponse)
        }
        switch handle {
        case .legacy(let handle):
            handle.startTask(completionCookie: completionCookie)
        case .task(let task):
            Task {
                do {
                    try completion?(try await task.value)
                } catch {
                    print("temp logging")
                }
            }
        }
    }

    public func cancel() {
        switch handle {
        case .legacy(let handle): handle.cancelTask()
        case .task(let task): task.cancel()
        }
    }
}
