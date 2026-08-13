//
//  SerialQueue.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-08-10.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import Foundation

public final class SerialQueue: Sendable {
    private typealias Block = @Sendable () async -> Void

    private let continuation: AsyncStream<Block>.Continuation
    private let worker: Task<Void, Never>

    deinit {
        continuation.finish()
        worker.cancel()
    }

    public init() {
        var stored: AsyncStream<Block>.Continuation!

        let stream = AsyncStream { continuation in
            stored = continuation
        }

        self.continuation = stored

        worker = Task.detached {
            for await operation in stream {
                await operation()
            }
        }
    }

    public func enqueue<T: Sendable>(
        completionQueue queue: DispatchQueue,
        _ operation: @escaping @Sendable () async throws -> T
    ) async throws -> T {
        let box = ContinuationBox<T>()

        return try await withTaskCancellationHandler {
            try await withCheckedThrowingContinuation { continuation in
                box.set(continuation)
                self.continuation.yield {
                    guard !box.isCompleted else { return }
                    do {
                        let result = try await operation()
                        queue.async {
                            box.resume(returning: .success(result))
                        }

                    } catch {
                        queue.async {
                            box.resume(returning: .failure(error))
                        }
                    }
                }
            }
        } onCancel: {
            queue.async {
                box.resume(returning: .failure(CancellationError()))
            }
        }
    }
}
