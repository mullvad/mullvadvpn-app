//
//  Cancellable.swift
//  MullvadTypes
//
//  Created by pronebird on 15/03/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

public protocol Cancellable {
    func cancel()
}

extension Operation: Cancellable {}

/// An object that can be used in place whenever a `Cancellable` is expected to be used.
///
/// Can be used to implement the concept of `no-op` or clean up tasks when an action has been cancelled.
public struct AnyCancellable: Cancellable {
    /// A block to call upon calling `cancel`.
    private let block: () -> Void

    public init(block: @escaping () -> Void) {
        self.block = block
    }

    /// Calls upon the `block` that was passed during init.
    public func cancel() {
        block()
    }
}
