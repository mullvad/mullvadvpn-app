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
    enum OutgoingConnectionResult {
        case ipV4(OutgoingConnectionProxy.IPV4ConnectionData)
        case ipV6(OutgoingConnectionProxy.IPV6ConnectionData)
    }

    private let outgoingConnectionProxy: OutgoingConnectionHandling

    init(outgoingConnectionProxy: OutgoingConnectionHandling) {
        self.outgoingConnectionProxy = outgoingConnectionProxy
    }

    func getOutgoingConnectionInfo() async throws -> OutgoingConnectionInfo {
        try await withThrowingTaskGroup(of: OutgoingConnectionResult.self) { taskGroup -> OutgoingConnectionInfo in

            taskGroup.addTask {
                let ipv4ConnectionInfo = try await self.outgoingConnectionProxy.getIPV4(retryStrategy: .default)
                return .ipV4(ipv4ConnectionInfo)
            }

            taskGroup.addTask {
                let ipv6ConnectionInfo = try await self.outgoingConnectionProxy.getIPV6(retryStrategy: .noRetry)
                return .ipV6(ipv6ConnectionInfo)
            }

            var ipv4 = OutgoingConnectionProxy.IPV4ConnectionData(ip: .any, exitIP: false)
            var ipv6 = OutgoingConnectionProxy.IPV6ConnectionData(ip: .any, exitIP: false)

            for try await value in taskGroup {
                switch value {
                case let .ipV4(connectionInfo):
                    ipv4 = connectionInfo
                case let .ipV6(connectionInfo):
                    ipv6 = connectionInfo
                }
            }

            return OutgoingConnectionInfo(ipv4: ipv4, ipv6: ipv6)
        }
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
