// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadREST

struct OutgoingConnectionProxyStub: OutgoingConnectionHandling {
    var ipV4: IPV4ConnectionData
    var ipV6: IPV6ConnectionData
    var error: Error?

    func getIPV6(retryStrategy: MullvadREST.REST.RetryStrategy) async throws -> IPV6ConnectionData {
        if let error {
            throw error
        } else {
            return ipV6
        }
    }

    func getIPV4(retryStrategy: MullvadREST.REST.RetryStrategy) async throws -> IPV4ConnectionData {
        if let error {
            throw error
        } else {
            return ipV4
        }
    }
}

extension IPV4ConnectionData {
    static let mock = IPV4ConnectionData(ip: .loopback, exitIP: true)
}

extension IPV6ConnectionData {
    static let mock = IPV6ConnectionData(ip: .loopback, exitIP: true)
}
