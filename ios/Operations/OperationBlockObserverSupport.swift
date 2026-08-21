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

public protocol OperationBlockObserverSupport {}

extension OperationBlockObserverSupport where Self: AsyncOperation {
    /// Add observer responding to cancellation event.
    public func onCancel(_ fn: @escaping (Self) -> Void) {
        addBlockObserver(OperationBlockObserver(didCancel: fn))
    }

    /// Add observer responding to finish event.
    public func onFinish(_ fn: @escaping (Self, Error?) -> Void) {
        addBlockObserver(OperationBlockObserver(didFinish: fn))
    }

    /// Add observer responding to start event.
    public func onStart(_ fn: @escaping (Self) -> Void) {
        addBlockObserver(OperationBlockObserver(didStart: fn))
    }

    /// Add block-based observer.
    public func addBlockObserver(_ observer: OperationBlockObserver<Self>) {
        addObserver(observer)
    }
}
