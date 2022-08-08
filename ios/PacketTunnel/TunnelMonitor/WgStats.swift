//
//  WgStats.swift
//  PacketTunnel
//
//  Created by pronebird on 08/08/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

struct WgStats {
    let bytesReceived: UInt64
    let bytesSent: UInt64

    init() {
        bytesReceived = 0
        bytesSent = 0
    }

    init?(from string: String) {
        var _bytesReceived: UInt64?
        var _bytesSent: UInt64?

        string.enumerateLines { line, stop in
            if _bytesReceived == nil, let value = parseValue("rx_bytes=", in: line) {
                _bytesReceived = value
            } else if _bytesSent == nil, let value = parseValue("tx_bytes=", in: line) {
                _bytesSent = value
            }

            if _bytesReceived != nil, _bytesSent != nil {
                stop = true
            }
        }

        guard let _bytesReceived = _bytesReceived, let _bytesSent = _bytesSent else {
            return nil
        }

        bytesReceived = _bytesReceived
        bytesSent = _bytesSent
    }
}

@inline(__always) private func parseValue(_ prefixKey: String, in line: String) -> UInt64? {
    guard line.hasPrefix(prefixKey) else { return nil }

    let value = line.dropFirst(prefixKey.count)

    return UInt64(value)
}
