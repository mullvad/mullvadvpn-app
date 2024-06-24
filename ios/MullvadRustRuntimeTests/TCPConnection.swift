//
//  TCPConnection.swift
//  TunnelObfuscationTests
//
//  Created by pronebird on 27/06/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network

/// Minimal implementation of TCP connection capable of receiving data.
/// > Warning: Do not use this implementation in production code. See the warning in `start()`.
class TCPConnection {
    private let dispatchQueue = DispatchQueue(label: "TCPConnection")
    private let nwConnection: NWConnection

    init(nwConnection: NWConnection) {
        self.nwConnection = nwConnection
    }

    deinit {
        cancel()
    }

    /// Establishes the TCP connection.
    ///
    /// > Warning: This implementation is **not safe to use in production**
    /// It will cancel the `listener.stateUpdateHandler` after it becomes ready and ignore future updates.
    ///
    /// Waits for the underlying connection to become ready before returning control to the caller, otherwise throws an
    /// error if connection state indicates as such.
    func start() async throws {
        try await withCheckedThrowingContinuation { continuation in
            nwConnection.stateUpdateHandler = { state in
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
                self.nwConnection.stateUpdateHandler = nil
            }
            nwConnection.start(queue: dispatchQueue)
        }
    }

    func cancel() {
        nwConnection.cancel()
    }

    func receiveData(minimumLength: Int, maximumLength: Int) async throws -> Data {
        return try await withCheckedThrowingContinuation { continuation in
            nwConnection.receive(
                minimumIncompleteLength: minimumLength,
                maximumLength: maximumLength
            ) { content, _, isComplete, error in
                if let error {
                    continuation.resume(throwing: error)
                } else if let content {
                    continuation.resume(returning: content)
                } else if isComplete {
                    continuation.resume(returning: Data())
                }
            }
        }
    }
}
