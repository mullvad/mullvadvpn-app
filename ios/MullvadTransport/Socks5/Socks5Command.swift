//
//  Socks5Command.swift
//  MullvadTransport
//
//  Created by pronebird on 21/10/2023.
//

import Foundation

/// Commands supported in socks protocol.
enum Socks5Command: UInt8 {
    case connect = 0x01
    case bind = 0x02
    case udpAssociate = 0x03
}
