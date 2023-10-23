//
//  Socks5ConnectNegotiation.swift
//  MullvadTransport
//
//  Created by pronebird on 20/10/2023.
//

import Foundation
import Network

/// The object handling a connection negotiation with socks proxy.
struct Socks5ConnectNegotiation {
    /// Connection to the socks proxy.
    let connection: NWConnection

    /// Endpoint to which the client wants to initiate connection over socks proxy.
    let endpoint: Socks5Endpoint

    /// Completion handler invoked on success.
    let onComplete: (Socks5ConnectReply) -> Void

    /// Failure handler invoked on error.
    let onFailure: (Error) -> Void

    /// Initiate negotiation by sending a connect command to the socks proxy.
    func perform() {
        let connectCommand = Socks5ConnectCommand(endpoint: endpoint)

        connection.send(content: connectCommand.rawData, completion: .contentProcessed { [self] error in
            if let error {
                onFailure(Socks5Error.remoteConnectionFailure(error))
            } else {
                readPartialReply()
            }
        })
    }

    /// Read the preamble of the connect reply.
    private func readPartialReply() {
        // The length of the preamble of the CONNECT reply.
        let replyPreambleLength = 4

        connection.receive(exactLength: replyPreambleLength) { [self] data, contentContext, isComplete, error in
            if let error {
                onFailure(Socks5Error.remoteConnectionFailure(error))
            } else if let data {
                do {
                    try handlePartialReply(data: data)
                } catch {
                    onFailure(error)
                }
            } else {
                onFailure(Socks5Error.unexpectedEndOfStream)
            }
        }
    }

    /**
     Parse the bytes that comprise the preamble of a connect reply and evaluate the status code. Upon success read the endpoint data to produce the complete
     reply and finish negotiation.

     The following fields are contained within the first 4 bytes: socks version, status code, reserved field, address type.
     */
    private func handlePartialReply(data: Data) throws {
        // Parse partial reply that contains the status code.
        let (statusCode, addressType) = try parsePartialReply(data: data)

        // Verify the status code.
        guard case .succeeded = statusCode else {
            throw Socks5Error.connectionRejected(statusCode)
        }

        // Parse server bound endpoint when partial reply indicates success.
        let endpointReader = Socks5EndpointReader(
            connection: connection,
            addressType: addressType,
            onComplete: { [self] endpoint in
                let reply = Socks5ConnectReply(status: statusCode, serverBoundEndpoint: endpoint)
                onComplete(reply)
            },
            onFailure: onFailure
        )
        endpointReader.perform()
    }

    /// Parse the bytes that comprise the preamble of reply without endpoint data.
    private func parsePartialReply(data: Data) throws -> (Socks5StatusCode, Socks5AddressType) {
        var iterator = data.makeIterator()

        // Read the protocol version.
        guard let version = iterator.next() else { throw Socks5Error.unexpectedEndOfStream }

        // Verify the protocol version.
        guard version == Socks5Constants.socksVersion else { throw Socks5Error.invalidSocksVersion }

        // Read status code, reserved field and address type from reply.
        guard let rawStatusCode = iterator.next(),
              iterator.next() != nil, // skip reserved field
              let rawAddressType = iterator.next() else {
            throw Socks5Error.unexpectedEndOfStream
        }

        // Parse the status code.
        guard let status = Socks5StatusCode(rawValue: rawStatusCode) else {
            throw Socks5Error.invalidStatusCode(rawStatusCode)
        }

        // Parse the address type.
        guard let addressType = Socks5AddressType(rawValue: rawAddressType) else {
            throw Socks5Error.invalidAddressType
        }

        return (status, addressType)
    }
}
