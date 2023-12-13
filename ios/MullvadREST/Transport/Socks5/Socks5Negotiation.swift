//
//  Socks5Negotiation.swift
//  MullvadREST
//
//  Created by Marco Nikic on 2023-12-13.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network

struct Socks5Authentication {
    let connection: NWConnection
    let endpoint: Socks5Endpoint
    let configuration: Socks5Configuration

    typealias AuthenticationComplete = () -> Void
    typealias AuthenticationFailure = (Error) -> Void

    func authenticate(onComplete: @escaping AuthenticationComplete, onFailure: @escaping AuthenticationFailure) {
        guard let username = configuration.username, let password = configuration.password else {
            onFailure(Socks5Error.invalidUsernameOrPassword)
            return
        }
        let authenticateCommand = Socks5UsernamePasswordCommand(username: username, password: password)

        connection.send(content: authenticateCommand.rawData, completion: .contentProcessed { maybeError in
            if let maybeError {
                onFailure(maybeError)
            } else {
                readNegotiationReply(onComplete: onComplete, onFailure: onFailure)
            }
        })
    }

    func readNegotiationReply(
        onComplete: @escaping AuthenticationComplete,
        onFailure: @escaping AuthenticationFailure
    ) {
        let replySize = MemoryLayout<Socks5UsernamePasswordReply>.size

        // Read in one shot, the payload is very small to not care about a reading loop.
        connection.receive(exactLength: replySize) { data, _, _, error in
            guard let data else {
                if let error {
                    onFailure(error)
                } else {
                    onFailure(Socks5Error.unexpectedEndOfStream)
                }
                return
            }

            guard let reply = Socks5UsernamePasswordReply(from: data) else {
                onFailure(Socks5Error.unexpectedEndOfStream)
                return
            }

            guard reply.version == Socks5Constants.usernamePasswordAuthenticationProtocol else {
                onFailure(Socks5Error.invalidSocksVersion)
                return
            }

            onComplete()
        }
    }
}
