//
//  Socks5AddressType.swift
//  MullvadTransport
//
//  Created by pronebird on 19/10/2023.
//

import Foundation

/// Address type supported by socks protocol
enum Socks5AddressType: UInt8 {
    case ipv4 = 0x01
    case domainName = 0x03
    case ipv6 = 0x04
}
