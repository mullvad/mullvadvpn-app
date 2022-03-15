//
//  Cancellable.swift
//  MullvadVPN
//
//  Created by pronebird on 15/03/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
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
        lock.lock()
        let block = closure
        closure = nil
        lock.unlock()

        block?()
    }
}
