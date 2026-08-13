//
//  ContinuationBox.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-08-13.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

final class ContinuationBox<T: Sendable>: @unchecked Sendable {
    private let lock = NSLock()
    var continuation: CheckedContinuation<T, Error>?

    var isCompleted: Bool {
        lock.lock()
        defer { lock.unlock() }
        return continuation == nil
    }

    func set(_ continuation: CheckedContinuation<T, Error>) {
        lock.lock()
        self.continuation = continuation
        lock.unlock()
    }

    func resume(returning value: Result<T, Error>) {
        lock.lock()
        let c = continuation
        continuation = nil
        lock.unlock()
        switch value {
        case .failure(let error):
            c?.resume(throwing: error)
        case .success(let value):
            c?.resume(returning: value)
        }
    }
}
