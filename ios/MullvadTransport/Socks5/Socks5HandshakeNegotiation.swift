//
//  Socks5HandshakeNegotiation.swift
//  MullvadTransport
//
//  Created by pronebird on 20/10/2023.
//

import Foundation
import Network

/// The object handling a handshake negotiation with socks proxy.
struct Socks5HandshakeNegotiation {
    let connection: NWConnection
    let handshake: Socks5Handshake
    let onComplete: (Socks5HandshakeReply) -> Void
    let onFailure: (Error) -> Void

    func perform() {
        connection.send(content: handshake.rawData, completion: .contentProcessed { [self] error in
            if let error {
                onFailure(Socks5Error.remoteConnectionFailure(error))
            } else {
                readReply()
            }
        })
    }

    private func readReply() {
        // The length of a handshake reply in bytes.
        let replyLength = 2

        connection.receive(exactLength: replyLength) { [self] data, context, isComplete, error in
            if let error {
                onFailure(Socks5Error.remoteConnectionFailure(error))
            } else if let data {
                do {
                    onComplete(try parseReply(data: data))
                } catch {
                    onFailure(error)
                }
            } else {
                onFailure(Socks5Error.unexpectedEndOfStream)
            }
        }
    }

    private func parseReply(data: Data) throws -> Socks5HandshakeReply {
        var iterator = data.makeIterator()

        guard let version = iterator.next() else { throw Socks5Error.unexpectedEndOfStream }
        guard version == Socks5Constants.socksVersion else { throw Socks5Error.invalidSocksVersion }

        guard let rawMethod = iterator.next() else { throw Socks5Error.unexpectedEndOfStream }

        // The response code returned by the server when none of the auth methods listed by the client are acceptable.
        let authMethodsUnacceptableReplyCode: UInt8 = 0xff

        guard rawMethod != authMethodsUnacceptableReplyCode else {
            throw Socks5Error.unacceptableAuthMethods
        }

        guard let method = Socks5AuthenticationMethod(rawValue: rawMethod) else {
            throw Socks5Error.unsupportedAuthMethod
        }

        return Socks5HandshakeReply(method: method)
    }
}
