//
//  Socks5StatusCode.swift
//  MullvadTransport
//
//  Created by pronebird on 19/10/2023.
//

import Foundation

/// Status code used in socks protocol.
public enum Socks5StatusCode: UInt8 {
    case succeeded = 0x00
    case failure = 0x01
    case connectionNotAllowedByRuleset = 0x02
    case networkUnreachable = 0x03
    case hostUnreachable = 0x04
    case connectionRefused = 0x05
    case ttlExpired = 0x06
    case commandNotSupported = 0x07
    case addressTypeNotSupported = 0x08
}
