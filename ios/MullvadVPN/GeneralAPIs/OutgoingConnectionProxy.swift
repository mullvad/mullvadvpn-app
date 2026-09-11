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
import MullvadREST
import MullvadRustRuntime
import MullvadTypes
import Network

protocol OutgoingConnectionHandling {
    func getIPV6(retryStrategy: REST.RetryStrategy) async throws -> IPV6ConnectionData
    func getIPV4(retryStrategy: REST.RetryStrategy) async throws -> IPV4ConnectionData
}

final class OutgoingConnectionProxy: OutgoingConnectionHandling {
    enum ExitIPVersion: String {
        case v4 = "ipv4"
        case v6 = "ipv6"

        func host(hostname: String) -> String {
            "\(rawValue).am.i.\(hostname)"
        }
    }

    let apiContext: ApiContext
    let hostname: String

    init(apiContext: ApiContext, hostname: String) {
        self.apiContext = apiContext
        self.hostname = hostname
    }

    func getIPV6(retryStrategy: REST.RetryStrategy) async throws -> IPV6ConnectionData {
        let data = try await apiContext.amIMullvad(
            useIpv6: true,
            hostname: hostname,
            retryStrategy: retryStrategy.toRustStrategy())
        guard let ipv6 = data.ipv6 else {
            throw APIError(statusCode: 0, errorDescription: "oops", serverResponseCode: nil)
        }
        return IPV6ConnectionData(ip: ipv6.toSwift(), exitIP: data.exitIp)
    }

    func getIPV4(retryStrategy: REST.RetryStrategy) async throws -> IPV4ConnectionData {
        let data = try await apiContext.amIMullvad(
            useIpv6: false,
            hostname: hostname,
            retryStrategy: retryStrategy.toRustStrategy())
        guard let ipv4 = data.ipv4 else {
            throw APIError(statusCode: 0, errorDescription: "oops", serverResponseCode: nil)
        }
        return IPV4ConnectionData(ip: ipv4.toSwift(), exitIP: data.exitIp)
    }
}
