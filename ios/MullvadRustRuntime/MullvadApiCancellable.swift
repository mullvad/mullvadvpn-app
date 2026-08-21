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

extension SwiftCancelHandle: @retroactive @unchecked Sendable {}

public final class MullvadApiCancellable: Cancellable, Sendable {
    private let handle: SwiftCancelHandle

    public init(handle: consuming SwiftCancelHandle) {
        self.handle = handle
    }

    public func start(_ completion: ((MullvadApiResponse) throws -> Void)?) {
        let completionPointer = MullvadApiCompletion { apiResponse in
            try? completion?(apiResponse)
        }
        let rawCompletionPointer = Unmanaged.passRetained(completionPointer).toOpaque()
        mullvad_api_start_task(handle, rawCompletionPointer)
    }

    deinit {
        mullvad_api_cancel_task_drop(handle)
    }

    public func cancel() {
        mullvad_api_cancel_task(handle)
    }
}
