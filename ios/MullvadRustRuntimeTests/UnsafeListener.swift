//
//  UnsafeListener.swift
//  MullvadRustRuntimeTests
//
//  Created by pronebird on 27/06/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Network

/// > Warning: Do not use this implementation in production code. See the warning in `start()`.
class UnsafeListener<T: Connection> {
    private let dispatchQueue = DispatchQueue(label: "com.test.unsafeListener")
    private let listener: NWListener

    /// A stream of new connections.
    /// The caller may iterate over this stream to accept new connections.
    ///
    /// `Connection` objects are returned unopen, so the caller has to call `Connection.start()` to accept the
    /// connection before initiating the data exchange.
    let newConnections: AsyncStream<T>

    init() throws {
        let listener = try NWListener(using: T.connectionParameters)

        newConnections = AsyncStream { continuation in
            listener.newConnectionHandler = { nwConnection in
                continuation.yield(T(nwConnection: nwConnection))
            }
            continuation.onTermination = { @Sendable _ in
                listener.newConnectionHandler = nil
            }
        }

        self.listener = listener
    }

    deinit {
        cancel()
    }

    /// Local port bound by listener on which it accepts new connections.
    var listenPort: UInt16 {
        return listener.port?.rawValue ?? 0
    }

    /// Start listening on a randomly assigned port which should be available via `listenPort` once this call returns
    /// control back to the caller.
    ///
    /// > Warning: This implementation is **not safe to use in production**
    /// It will cancel the `listener.stateUpdateHandler` after it becomes ready and ignore future updates.
    ///
    /// Waits for the underlying connection to become ready before returning control to the caller, otherwise throws an
    /// error if connection state indicates as such.
    func start() async throws {
        try await withCheckedThrowingContinuation { continuation in
            listener.stateUpdateHandler = { state in
                switch state {
                case .ready:
                    continuation.resume(returning: ())
                case let .failed(error):
                    continuation.resume(throwing: error)
                case let .waiting(error):
                    continuation.resume(throwing: error)
                case .cancelled:
                    continuation.resume(throwing: CancellationError())
                default:
                    return
                }
                // Reset state update handler after resuming continuation.
                self.listener.stateUpdateHandler = nil
            }
            listener.start(queue: dispatchQueue)
        }
    }

    func cancel() {
        listener.cancel()
    }
}
