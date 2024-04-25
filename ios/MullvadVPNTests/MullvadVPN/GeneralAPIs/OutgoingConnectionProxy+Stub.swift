//
//  File.swift
//  MullvadVPNTests
//
//  Created by Mojgan on 2023-10-25.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

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
