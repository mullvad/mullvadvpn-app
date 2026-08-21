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

import protocol MullvadTypes.Cancellable

/// Asynchronous block operation
public class AsyncBlockOperation: AsyncOperation, @unchecked Sendable {
    private var executor: ((@escaping @Sendable (Error?) -> Void) -> Cancellable?)?
    private var cancellableTask: Cancellable?

    public init(
        dispatchQueue: DispatchQueue? = nil,
        block: @escaping @Sendable (@escaping @Sendable (Error?) -> Void) -> Void
    ) {
        super.init(dispatchQueue: dispatchQueue)
        executor = { finish in
            block(finish)
            return nil
        }
    }

    public init(dispatchQueue: DispatchQueue? = nil, block: @escaping @Sendable () -> Void) {
        super.init(dispatchQueue: dispatchQueue)
        executor = { finish in
            block()
            finish(nil)
            return nil
        }
    }

    public init(
        dispatchQueue: DispatchQueue? = nil,
        cancellableTask: @escaping @Sendable (@escaping @Sendable (Error?) -> Void) -> Cancellable
    ) {
        super.init(dispatchQueue: dispatchQueue)
        executor = { cancellableTask($0) }
    }

    override public func main() {
        let executor = executor
        self.executor = nil

        assert(executor != nil)

        cancellableTask = executor?(self.finish)
    }

    override public func operationDidCancel() {
        cancellableTask?.cancel()
    }

    override public func operationDidFinish() {
        executor = nil
        cancellableTask = nil
    }
}
