//
//  Cancellable.swift
//  MullvadVPN
//
//  Created by pronebird on 23/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol Cancellable {
    func cancel()
}

class AnyCancellable: Cancellable {
    private var closure: (() -> Void)?
    private let lock = NSLock()

    init(_ block: @escaping () -> Void) {
        self.closure = block
    }

    func cancel() {
        lock.withCriticalBlock {
            self.closure?()
            self.closure = nil
        }
    }
}
