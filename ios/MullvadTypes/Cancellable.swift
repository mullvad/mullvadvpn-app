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

public protocol Cancellable {
    func cancel()
}

extension Operation: Cancellable {}

/// An object representing a cancellation token.
public final class AnyCancellable: Cancellable {
    private let block: (() -> Void)?

    /// Create cancellation token with block handler.
    public init(block: @escaping @Sendable () -> Void) {
        self.block = block
    }

    /// Create empty cancellation token.
    public init() {
        block = nil
    }

    public func cancel() {
        block?()
    }
}
