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

public final class AnyCancellable: Cancellable {
    private let block: () -> Void

    public init(block: @escaping () -> Void) {
        self.block = block
    }

    public func cancel() {
        block()
    }
}
