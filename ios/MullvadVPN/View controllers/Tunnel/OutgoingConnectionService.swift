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
    func getOutgoingIPv4ConnectionInfo() async throws -> IPV4ConnectionData
    func getOutgoingIPv6ConnectionInfo() async throws -> IPV6ConnectionData
}

final class OutgoingConnectionService: OutgoingConnectionServiceHandling {
    private let outgoingConnectionProxy: OutgoingConnectionHandling

    init(outgoingConnectionProxy: OutgoingConnectionHandling) {
        self.outgoingConnectionProxy = outgoingConnectionProxy
    }

    func getOutgoingIPv4ConnectionInfo() async throws -> IPV4ConnectionData {
        try await outgoingConnectionProxy.getIPV4(retryStrategy: .default)
    }
    func getOutgoingIPv6ConnectionInfo() async throws -> IPV6ConnectionData {
        try await outgoingConnectionProxy.getIPV6(retryStrategy: .default)
    }
}

struct OutgoingConnectionInfo {
    /// IPv4  exit connection.
    var ipv4: IPV4ConnectionData?

    /// IPv6 exit connection.
    var ipv6: IPV6ConnectionData?

    var outAddress: String? {
        var ipv4String: String?
        if let ipv4 = ipv4, ipv4.exitIP {
            ipv4String = "\(ipv4.ip)"
        }

        var ipv6String: String?
        if let ipv6 = ipv6, ipv6.exitIP {
            ipv6String = "\(ipv6.ip)"
        }

        let outAddress = [ipv4String, ipv6String].compactMap { $0 }.joined(separator: "\n")
        return outAddress.isEmpty ? nil : outAddress
    }
}
