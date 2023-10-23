//
//  Socks5DataStreamHandler.swift
//  MullvadTransport
//
//  Created by pronebird on 20/10/2023.
//

import Foundation
import Network

/// The object handling bidirectional streaming of data between local and remote connection.
struct Socks5DataStreamHandler {
    /// How many bytes the handler can receive at one time, when streaming data between local and remote connection.
    static let maxBytesToRead = Int(UInt16.max)

    /// Local TCP connection.
    let localConnection: NWConnection

    /// Remote TCP connection to the socks proxy.
    let remoteConnection: NWConnection

    /// Error handler.
    let errorHandler: (Error) -> Void

    /// Start streaming data between local and remote connection.
    func start() {
        streamOutboundTraffic()
        streamInboundTraffic()
    }

    /// Pass outbound traffic from local to remote connection.
    private func streamOutboundTraffic() {
        localConnection.receive(
            minimumIncompleteLength: 1,
            maximumLength: Self.maxBytesToRead
        ) { [self] content, _, isComplete, error in
            if let error {
                errorHandler(Socks5Error.localConnectionFailure(error))
                return
            }

            remoteConnection.send(
                content: content,
                isComplete: isComplete,
                completion: .contentProcessed { [self] error in
                    if let error {
                        errorHandler(Socks5Error.remoteConnectionFailure(error))
                    } else if !isComplete {
                        streamOutboundTraffic()
                    }
                }
            )
        }
    }

    /// Pass inbound traffic from remote to local connection.
    private func streamInboundTraffic() {
        remoteConnection.receive(
            minimumIncompleteLength: 1,
            maximumLength: Self.maxBytesToRead
        ) { [self] content, _, isComplete, error in
            if let error {
                errorHandler(Socks5Error.remoteConnectionFailure(error))
                return
            }

            localConnection.send(
                content: content,
                isComplete: isComplete,
                completion: .contentProcessed { [self] error in
                    if let error {
                        errorHandler(Socks5Error.localConnectionFailure(error))
                    } else if !isComplete {
                        streamInboundTraffic()
                    }
                }
            )
        }
    }
}
