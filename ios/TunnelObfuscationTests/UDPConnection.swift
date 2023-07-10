//
//  UDPConnection.swift
//  TunnelObfuscationTests
//
//  Created by pronebird on 27/06/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network

/// Minimal implementation of UDP connection capable of sending data.
/// > Warning: Do not use this implementation in production code. See the warning in `start()`.
class UDPConnection {
    private let dispatchQueue = DispatchQueue(label: "UDPConnection")
    private let nwConnection: NWConnection

    init(remote: IPAddress, port: UInt16) {
        nwConnection = NWConnection(
            host: NWEndpoint.Host("\(remote)"),
            port: NWEndpoint.Port(integerLiteral: port),
            using: .udp
        )
    }

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

    func sendData(_ data: Data) async throws {
        return try await withCheckedThrowingContinuation { continuation in
            nwConnection.send(content: data, completion: .contentProcessed { error in
                if let error {
                    continuation.resume(throwing: error)
                } else {
                    continuation.resume(returning: ())
                }
            })
        }
    }
}
