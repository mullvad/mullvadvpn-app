//
//  AsyncSerialExecutor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-08-10.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

public final class AsyncSerialExecutor: Sendable {
    private typealias Operation = @Sendable () async -> Void

    private let continuation: AsyncStream<Operation>.Continuation

    deinit {
        continuation.finish()
    }

    public init() {
        var stored: AsyncStream<Operation>.Continuation!

        let stream = AsyncStream { continuation in
            stored = continuation
        }

        self.continuation = stored

        Task {
            for await operation in stream {
                await operation()
            }
        }
    }

    public func enqueue<T: Sendable>(
        _ operation: @escaping @Sendable () async throws -> T
    ) async throws -> T {
        try await withCheckedThrowingContinuation {
            (continuation: CheckedContinuation<T, Error>) in

            self.continuation.yield {
                do {
                    let result = try await operation()
                    continuation.resume(returning: result)
                } catch {
                    continuation.resume(throwing: error)
                }
            }
        }
    }
}
