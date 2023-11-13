//
//  File.swift
//  MullvadVPNTests
//
//  Created by Mojgan on 2023-10-25.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadREST

struct OutgoingConnectionProxyStub: OutgoingConnectionHandling {
    var hasError = false
    var ipV4: OutgoingConnectionProxy.IPV4ConnectionData
    var ipV6: OutgoingConnectionProxy.IPV6ConnectionData
    var error: Error

    func getIPV6(retryStrategy: MullvadREST.REST.RetryStrategy) async throws -> OutgoingConnectionProxy
        .IPV6ConnectionData {
        if hasError {
            throw error
        } else {
            return ipV6
        }
    }

    func getIPV4(retryStrategy: MullvadREST.REST.RetryStrategy) async throws -> OutgoingConnectionProxy
        .IPV4ConnectionData {
        if hasError {
            throw error
        } else {
            return ipV4
        }
    }
}

extension OutgoingConnectionProxy.IPV4ConnectionData {
    static let mock = OutgoingConnectionProxy.IPV4ConnectionData(ip: .loopback, exitIP: true)
}

extension OutgoingConnectionProxy.IPV6ConnectionData {
    static let mock = OutgoingConnectionProxy.IPV6ConnectionData(ip: .loopback, exitIP: true)
}
