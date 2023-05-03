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

/// An object representing a cancellation token.
public final class AnyCancellable: Cancellable {
    private let block: (() -> Void)?

    /// Create cancellation token with block handler.
    public init(block: @escaping () -> Void) {
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
