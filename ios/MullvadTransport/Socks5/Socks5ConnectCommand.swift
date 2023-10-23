//
//  Socks5ConnectCommand.swift
//  MullvadTransport
//
//  Created by pronebird on 19/10/2023.
//

import Foundation
import Network

/// The connect command message.
struct Socks5ConnectCommand {
    /// The remote endpoint to which the client wants to establish connection over the socks proxy.
    var endpoint: Socks5Endpoint

    /// The byte representation in socks protocol.
    var rawData: Data {
        var data = Data()

        // Socks version.
        data.append(Socks5Constants.socksVersion)

        // Command code.
        data.append(Socks5Command.connect.rawValue)

        // Reserved.
        data.append(0)

        // Address type.
        data.append(endpoint.addressType.rawValue)

        // Endpoint address.
        data.append(endpoint.rawData)

        return data
    }
}

/// The connect command reply message.
struct Socks5ConnectReply {
    /// The server status code.
    var status: Socks5StatusCode

    /// The server bound endpoint.
    var serverBoundEndpoint: Socks5Endpoint
}
