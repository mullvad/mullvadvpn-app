//
//  DeferredCancellable.swift
//  MullvadTransport
//
//  Created by pronebird on 23/10/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

/// Cancellable object that defers cancellation until the other token is connected to it.
final class DeferredCancellable: Cancellable {
    private let stateLock = NSLock()
    private var isCancelled = false
    private var connectedTokens: [Cancellable] = []

    init() {}

    /// Connect deferred cancellation token with some other.
    ///
    /// The token is cancelled immediately, if the deferred object is already cancelled.
    func connect(_ token: Cancellable) {
        stateLock.withLock {
            if isCancelled {
                token.cancel()
            } else {
                connectedTokens.append(token)
            }
        }
    }

    /// Request cancellation.
    ///
    /// Cancels and releases any of the connected tokens.
    func cancel() {
        stateLock.withLock {
            isCancelled = true
            connectedTokens.forEach { $0.cancel() }
            connectedTokens.removeAll()
        }
    }
}
