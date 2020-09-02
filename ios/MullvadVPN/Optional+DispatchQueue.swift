//
//  Optional+DispatchQueue.swift
//  MullvadVPN
//
//  Created by pronebird on 01/09/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension Optional where Wrapped == DispatchQueue {
    /// Unwrap the `DispatchQueue` and perform the block on it, otherwise call the block
    /// synchronously on the current queue when `Optional` is `.none`.
    func performOnWrappedOrCurrentQueue(block: @escaping () -> Void) {
        switch self {
        case .some(let queue):
            queue.async(execute: block)
        case .none:
            block()
        }
    }
}
