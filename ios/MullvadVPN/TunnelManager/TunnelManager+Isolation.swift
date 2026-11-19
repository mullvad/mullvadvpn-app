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

// This hack should be removed once we've moved to iOS 18. This only exists because the runtime gets confused about isolation
// on iOS 17 due to incomplete API that was fixed in iOS 18.
// See: https://github.com/swiftlang/swift-evolution/blob/main/proposals/0424-custom-isolation-checking-for-serialexecutor.md#motivation
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
                // Stolen from: https://github.com/swiftlang/swift/blob/b119cd09e9c6e334a36fdf256340a55efda6844f/stdlib/public/Concurrency/ExecutorAssertions.swift#L336
                let isolationStripped = unsafeBitCast(escapableBlock, to: NonIsolatedSelf.self)
                return try isolationStripped(self)
            }
        }
    }
}
