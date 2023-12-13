//
//  Socks5UsernamePasswordCommand.swift
//  MullvadREST
//
//  Created by Marco Nikic on 2023-12-13.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/**
 The payload sent to the server is the following diagram
 +----+------+----------+------+----------+
 |VER | ULEN |  UNAME   | PLEN |  PASSWD  |
 +----+------+----------+------+----------+
 | 1  |  1   | 1 to 255 |  1   | 1 to 255 |
 +----+------+----------+------+----------+

 VER: The current version of this method, always 1
 ULEN: The length of `username`
 UNAME: The username
 PLEN: The length of `password`
 PASSWD: The password

 **/
struct Socks5UsernamePasswordCommand {
    let username: String
    let password: String

    var rawData: Data {
        var data = Data()
        guard username.count < UInt8.max,
              password.count < UInt8.max,
              let usernameData = username.data(using: .utf8),
              let passwordData = password.data(using: .utf8)
        else { return data }

        // Protocol version
        data.append(Socks5Constants.usernamePasswordAuthenticationProtocol)

        // Username length
        data.append(UInt8(username.count))

        // Username
        data.append(usernameData)

        // Password length
        data.append(UInt8(password.count))

        // Password
        data.append(passwordData)

        return data
    }
}

/**
 The expected answer payload looks like this
 +-----+--------+
 | VER | STATUS |
 +-----+--------+
 | 1   |   1    |
 +-----+--------+
 */
struct Socks5UsernamePasswordReply {
    let version: UInt8
    let status: Socks5StatusCode

    /// - Parameter data: The bytes read from the network connection sent by a socks5 server as a reply to a `Socks5UsernamePasswordCommand`.
    init?(from data: Data) {
        let expectedSize = MemoryLayout<Self>.size
        guard data.count == expectedSize else { return nil }
        var iterator = data.makeIterator()

        guard let readVersion = iterator.next() else { return nil }
        guard readVersion == Socks5Constants.usernamePasswordAuthenticationProtocol else { return nil }
        self.version = readVersion

        guard let readStatus = iterator.next(),
              let statusCode = Socks5StatusCode(rawValue: readStatus) else { return nil }
        self.status = statusCode
    }
}
