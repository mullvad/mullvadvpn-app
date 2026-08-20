//
//  Promise.swift
//  MullvadVPN
//
//  Created by pronebird on 28/01/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

// This object can be used like an async semaphore with exactly 1 writer. It
// allows the waiter to wait to `receive()` from another operation
// asynchronously. It is important not to forget to call `send`, otherwise this
// operation will block indefinitely.
public struct OneshotChannel: Sendable {
    private var continuation: AsyncStream<Void>.Continuation?
    private var stream: AsyncStream<Void>

    public init() {
        var ownedContinuation: AsyncStream<Void>.Continuation?
        stream = AsyncStream { continuation in
            ownedContinuation = continuation
        }
        self.continuation = ownedContinuation
    }

    public func send() {
        continuation?.yield()
        continuation?.finish()
    }

    public func receive() async {
        for await _ in stream {
            return
        }
    }
}
