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

extension TunnelManager {
    nonisolated func assumeIsolatedHack<T>(
        _ block: (isolated TunnelManager) throws -> T
    ) rethrows -> T where T: Sendable {
        typealias NonIsolatedSelf = (TunnelManager) throws -> T

        if #available(iOS 18.0, *) {
            return try assumeIsolated(block)
        } else {
            dispatchPrecondition(condition: .onQueue(internalQueue))
            return try withoutActuallyEscaping(block) { escapableBlock in
                // bitcast self from an isolated declaration to a non-isolated declaration
                // Stolen from : https://github.com/swiftlang/swift/blob/b119cd09e9c6e334a36fdf256340a55efda6844f/stdlib/public/Concurrency/ExecutorAssertions.swift#L336
                let isolationStripped = unsafeBitCast(escapableBlock, to: NonIsolatedSelf.self)
                return try isolationStripped(self)
            }
        }
    }
}
