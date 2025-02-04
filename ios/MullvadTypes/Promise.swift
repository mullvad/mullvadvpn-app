//
//  Promise.swift
//  MullvadVPN
//
//  Created by pronebird on 28/01/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

public final class Promise<Success, Failure: Error>: @unchecked Sendable {
    public typealias Result = Swift.Result<Success, Failure>

    private let nslock = NSLock()
    private var observers: [(Result) -> Void] = []
    private var result: Result?

    public init(_ executor: (@escaping (Result) -> Void) -> Void) {
        executor(resolve)
    }

    public func observe(_ completion: @escaping (Result) -> Void) {
        nslock.lock()
        if let result {
            nslock.unlock()
            completion(result)
        } else {
            observers.append(completion)
            nslock.unlock()
        }
    }

    private func resolve(result: Result) {
        nslock.lock()
        if self.result == nil {
            self.result = result

            let observers = observers
            self.observers.removeAll()
            nslock.unlock()

            for observer in observers {
                observer(result)
            }
        } else {
            nslock.unlock()
        }
    }
}

// This object can be used like an async semaphore with exactly 1 writer. It
// allows the waiter to wait to `receive()` from another operation
// asynchronously. It is important not to forget to call `send`, otherwise this
// operation will block indefinitely.
public struct OneshotChannel {
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
