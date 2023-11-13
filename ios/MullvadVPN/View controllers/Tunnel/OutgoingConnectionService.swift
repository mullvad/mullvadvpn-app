//
//  OutgoingConnectionService.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-10-27.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import Network

protocol OutgoingConnectionServiceHandling {
    func getOutgoingConnectionInfo() async throws -> OutgoingConnectionInfo
}

final class OutgoingConnectionService: OutgoingConnectionServiceHandling {
    private let outgoingConnectionProxy: OutgoingConnectionHandling

    init(outgoingConnectionProxy: OutgoingConnectionHandling) {
        self.outgoingConnectionProxy = outgoingConnectionProxy
    }

    func getOutgoingConnectionInfo() async throws -> OutgoingConnectionInfo {
        let ipv4ConnectionInfo = try await outgoingConnectionProxy.getIPV4(retryStrategy: .default)
        let ipv6ConnectionInfo = try await outgoingConnectionProxy.getIPV6(retryStrategy: .noRetry)
        return OutgoingConnectionInfo(ipv4: ipv4ConnectionInfo, ipv6: ipv6ConnectionInfo)
    }
}

struct OutgoingConnectionInfo {
    /// IPv4  exit connection.
    let ipv4: OutgoingConnectionProxy.IPV4ConnectionData

    /// IPv6 exit connection.
    let ipv6: OutgoingConnectionProxy.IPV6ConnectionData

    var outAddress: String? {
        let v4 = ipv4.exitIP ? "\(ipv4.ip)" : nil
        let v6 = ipv6.exitIP ? "\(ipv6.ip)" : nil
        let outAddress = [v4, v6].compactMap { $0 }.joined(separator: "\n")
        return outAddress.isEmpty ? nil : outAddress
    }
}
