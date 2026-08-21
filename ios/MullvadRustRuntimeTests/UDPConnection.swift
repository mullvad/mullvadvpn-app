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

protocol Connection {
    init(nwConnection: NWConnection)
    static var connectionParameters: NWParameters { get }
}

/// Minimal implementation of UDP connection capable of sending data.
/// > Warning: Do not use this implementation in production code. See the warning in `start()`.
class UDPConnection: Connection, @unchecked Sendable {
    private let dispatchQueue = DispatchQueue(label: "UDPConnection")
    private let nwConnection: NWConnection

    convenience init(remote: IPAddress, port: UInt16) {
        self.init(
            nwConnection: NWConnection(
                host: NWEndpoint.Host("\(remote)"),
                port: NWEndpoint.Port(integerLiteral: port),
                using: .udp
            ))
    }

    required init(nwConnection: NWConnection) {
        self.nwConnection = nwConnection
    }

    static var connectionParameters: NWParameters { .udp }

    deinit {
        cancel()
    }

    /// Establishes the UDP connection.
    ///
    /// > Warning: This implementation is **not safe to use in production**
    /// It will cancel the `listener.stateUpdateHandler` after it becomes ready and ignore future updates.
    ///
    /// Waits for the underlying connection to become ready before returning control to the caller, otherwise throws an
    /// error if connection state indicates as such.
    func start() async throws {
        return try await withCheckedThrowingContinuation { continuation in
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

    func readSingleDatagram() async throws -> Data {
        return try await withCheckedThrowingContinuation { continuation in
            nwConnection.receiveMessage { data, _, _, error in
                guard let data else {
                    continuation.resume(throwing: POSIXError(.EIO))
                    return
                }
                if let error {
                    continuation.resume(throwing: error)
                    return
                }
                continuation.resume(with: .success(data))
            }
        }
    }

    func sendData(_ data: Data) async throws {
        return try await withCheckedThrowingContinuation { continuation in
            nwConnection.send(
                content: data,
                completion: .contentProcessed { error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else {
                        continuation.resume(returning: ())
                    }
                })
        }
    }
}
