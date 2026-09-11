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
    let apiContext: ApiContext
    let hostname: String

    init(
        apiContext: ApiContext,
        hostname: String = REST.amIMullvadHostname,
    ) {
        self.apiContext = apiContext
        self.hostname = hostname
    }

    func getIPV6(retryStrategy: REST.RetryStrategy) async throws -> IPV6ConnectionData {
        guard
            let data = await apiContext.amIMullvad(
                address: "https://ipv6.\(hostname)/json",
                retryStrategy: retryStrategy.toRustStrategy()),
            case let .ipv6(ipv6) = data.ip
        else {
            throw APIError(statusCode: 0, errorDescription: "getting ipv6 address failed", serverResponseCode: nil)
        }
        return IPV6ConnectionData(ip: ipv6, exitIP: data.mullvadExitIp)
    }

    func getIPV4(retryStrategy: REST.RetryStrategy) async throws -> IPV4ConnectionData {
        guard
            let data = await apiContext.amIMullvad(
                address: "https://ipv4.\(hostname)/json",
                retryStrategy: retryStrategy.toRustStrategy()),
            case let .ipv4(ipv4) = data.ip
        else {
            throw APIError(statusCode: 0, errorDescription: "getting ipv4 address failed", serverResponseCode: nil)
        }
        return IPV4ConnectionData(ip: ipv4, exitIP: data.mullvadExitIp)
    }
}
