//
//  TCPListener.swift
//  TunnelObfuscationTests
//
//  Created by pronebird on 27/06/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Network

/// Minimal implementation of a TCP listener.
class TCPOneShotListener {
    private let dispatchQueue = DispatchQueue(label: "TCPListener")
    private let listener: NWListener

    /// A stream of new TCP connections.
    /// The caller may iterate over this stream to accept new TCP connections.
    ///
    /// `TCPConnection` objects are returned unopen, so the caller has to call `TCPConnection.start()` to accept the
    /// connection before initiating the data exchange.
    let newConnections: AsyncStream<TCPConnection>

    init() throws {
        let listener = try NWListener(using: .tcp)

        newConnections = AsyncStream { continuation in
            listener.newConnectionHandler = { nwConnection in
                continuation.yield(TCPConnection(nwConnection: nwConnection))
            }
            continuation.onTermination = { _ in
                listener.newConnectionHandler = nil
            }
        }

        self.listener = listener
    }

    deinit {
        cancel()
    }

    /// Local TCP port bound by listener on which it accepts new connections.
    var listenPort: UInt16 {
        return listener.port?.rawValue ?? 0
    }

    /// Start listening on a randomly assigned port which should be available via `listenPort` once this call returns
    /// control back to the caller.
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
