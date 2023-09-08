//
//  OperationBlockObserverSupport.swift
//  Operations
//
//  Created by Jon Petersson on 2023-09-07.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

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
