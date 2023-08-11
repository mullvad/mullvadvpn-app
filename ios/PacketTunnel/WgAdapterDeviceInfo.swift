//
//  WgAdapterInfoProvider.swift
//  PacketTunnel
//
//  Created by pronebird on 08/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import PacketTunnelCore
import WireGuardKit

struct WgAdapterDeviceInfo: TunnelDeviceInfoProtocol {
    let adapter: WireGuardAdapter

    var interfaceName: String? {
        return adapter.interfaceName
    }

    func getStats() throws -> WgStats {
        var result: String?

        let dispatchGroup = DispatchGroup()
        dispatchGroup.enter()
        adapter.getRuntimeConfiguration { string in
            result = string
            dispatchGroup.leave()
        }

        guard case .success = dispatchGroup.wait(wallTimeout: .now() + DispatchTimeInterval.seconds(1))
        else { throw StatsError.timeout }
        guard let result else { throw StatsError.nilValue }
        guard let newStats = WgStats(from: result) else { throw StatsError.parse }

        return newStats
    }

    enum StatsError: LocalizedError {
        case timeout, nilValue, parse

        var errorDescription: String? {
            switch self {
            case .timeout:
                return "adapter.getRuntimeConfiguration timeout."
            case .nilValue:
                return "Received nil string for stats."
            case .parse:
                return "Couldn't parse stats."
            }
        }
    }
}

private extension WgStats {
    init?(from string: String) {
        var bytesReceived: UInt64?
        var bytesSent: UInt64?

        string.enumerateLines { line, stop in
            if bytesReceived == nil, let value = parseValue("rx_bytes=", in: line) {
                bytesReceived = value
            } else if bytesSent == nil, let value = parseValue("tx_bytes=", in: line) {
                bytesSent = value
            }

            if bytesReceived != nil, bytesSent != nil {
                stop = true
            }
        }

        guard let bytesReceived, let bytesSent else {
            return nil
        }

        self.init(bytesReceived: bytesReceived, bytesSent: bytesSent)
    }
}

@inline(__always) private func parseValue(_ prefixKey: String, in line: String) -> UInt64? {
    guard line.hasPrefix(prefixKey) else { return nil }

    let value = line.dropFirst(prefixKey.count)

    return UInt64(value)
}
