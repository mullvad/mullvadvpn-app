//
//  CancellableChain.swift
//  MullvadTransport
//
//  Created by pronebird on 23/10/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

/// Cancellable object that cancels all cancellable objects linked to it.
final class CancellableChain: Cancellable {
    private let stateLock = NSLock()
    private var isCancelled = false
    private var linkedTokens: [Cancellable] = []

    init() {}

    /// Link cancellation token with some other.
    ///
    /// The token is cancelled immediately, if the chain is already cancelled.
    func link(_ token: Cancellable) {
        stateLock.withLock {
            if isCancelled {
                token.cancel()
            } else {
                linkedTokens.append(token)
            }
        }
    }

    /// Request cancellation.
    ///
    /// Cancels and releases any of the connected tokens.
    func cancel() {
        stateLock.withLock {
            isCancelled = true
            linkedTokens.forEach { $0.cancel() }
            linkedTokens.removeAll()
        }
    }
}
