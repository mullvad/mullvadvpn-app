// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
        let ipv6ConnectionInfo = try? await outgoingConnectionProxy.getIPV6(retryStrategy: .noRetry)
        return OutgoingConnectionInfo(ipv4: ipv4ConnectionInfo, ipv6: ipv6ConnectionInfo)
    }
}

struct OutgoingConnectionInfo {
    /// IPv4  exit connection.
    let ipv4: IPV4ConnectionData

    /// IPv6 exit connection.
    let ipv6: IPV6ConnectionData?

    var outAddress: String? {
        let ipv4String = ipv4.exitIP ? "\(ipv4.ip)" : nil

        var ipv6String: String?
        if let ipv6 = ipv6, ipv6.exitIP {
            ipv6String = "\(ipv6.ip)"
        }

        let outAddress = [ipv4String, ipv6String].compactMap { $0 }.joined(separator: "\n")
        return outAddress.isEmpty ? nil : outAddress
    }
}
