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
    private let handle: RequestCancelHandle

    public init(handle: consuming RequestCancelHandle) {
        self.handle = handle
    }

    public func start(_ completion: (@Sendable (ApiResponse) throws -> Void)?) {
        let completionCookie = MullvadApiCompletion { apiResponse in
            try? completion?(apiResponse)
        }
        handle.startTask(completionCookie: completionCookie)
    }

    public func cancel() {
        handle.cancelTask()
    }
}
