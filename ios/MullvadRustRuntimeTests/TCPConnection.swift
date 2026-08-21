// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import Network

/// Minimal implementation of TCP connection capable of receiving data.
/// > Warning: Do not use this implementation in production code. See the warning in `start()`.
class TCPConnection: Connection, @unchecked Sendable {
    private let dispatchQueue = DispatchQueue(label: "TCPConnection")
    private let nwConnection: NWConnection

    required init(nwConnection: NWConnection) {
        self.nwConnection = nwConnection
    }

    static var connectionParameters: NWParameters { .tcp }

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
