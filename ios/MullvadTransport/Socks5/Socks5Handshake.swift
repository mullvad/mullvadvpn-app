//
//  Socks5Handshake.swift
//  MullvadTransport
//
//  Created by pronebird on 19/10/2023.
//

import Foundation

/// Handshake initiation message.
struct Socks5Handshake {
    /// Authentication methods supported by the client.
    /// Defaults to `.notRequired` when empty.
    var methods: [Socks5AuthenticationMethod] = []

    /// The byte representation in socks protocol.
    var rawData: Data {
        var data = Data()
        var methods = methods

        // Make sure to provide at least one supported authentication method.
        if methods.isEmpty {
            methods.append(.notRequired)
        }

        // Append socks version
        data.append(Socks5Constants.socksVersion)

        // Append number of suppported authentication methods supported.
        data.append(UInt8(methods.count))

        // Append authentication methods
        data.append(contentsOf: methods.map { $0.rawValue })

        return data
    }
}

/// Handshake reply message.
struct Socks5HandshakeReply {
    /// The authentication method accepted by the socks proxys.
    var method: Socks5AuthenticationMethod
}
