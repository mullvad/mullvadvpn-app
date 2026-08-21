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

/**
 Type that cancels the task held inside upon `deinit`.

 It behaves identical to `Combine.AnyCancellable`.
 */
public final class AutoCancellingTask: Sendable {
    private let task: AnyTask

    init(_ task: AnyTask) {
        self.task = task
    }

    deinit {
        task.cancel()
    }
}
