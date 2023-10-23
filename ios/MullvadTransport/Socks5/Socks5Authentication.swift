//
//  Socks5Authentication.swift
//  MullvadTransport
//
//  Created by pronebird on 19/10/2023.
//

import Foundation

/// Authentication methods supported by socks protocol.
enum Socks5AuthenticationMethod: UInt8 {
    case notRequired = 0x00
    case usernamePassword = 0x02
}
