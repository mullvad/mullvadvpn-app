// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Network
import XCTest

@testable import MullvadMockData
@testable import MullvadREST
@testable import MullvadRustRuntime
@testable import MullvadSettings
@testable import MullvadTypes

final class OutgoingConnectionProxyTests: XCTestCase {
    private var mockIPV6ConnectionData: Data!
    private var mockIPV4ConnectionData: Data!

    private let encoder = JSONEncoder()

    override func setUpWithError() throws {
        mockIPV4ConnectionData = try encoder.encode(IPV4ConnectionData.mock)
        mockIPV6ConnectionData = try encoder.encode(IPV6ConnectionData.mock)
    }

    override func tearDownWithError() throws {
        mockIPV4ConnectionData.removeAll()
        mockIPV6ConnectionData.removeAll()
    }

    func testSuccessGettingIPV4() async throws {
        let outgoingConnectionProxy = OutgoingConnectionProxy(
            apiContext: try makeApiContext(disableTls: false)
        )

        _ = try await outgoingConnectionProxy.getIPV4(retryStrategy: .noRetry)
    }

    func testFailureGettingIPV4() async throws {
        let noIPv4Expectation = expectation(description: "Did not receive IPv4")

        let outgoingConnectionProxy = OutgoingConnectionProxy(
            apiContext: try makeApiContext(disableTls: true),
            hostname: "localhost:1")

        await XCTAssertThrowsErrorAsync(try await outgoingConnectionProxy.getIPV4(retryStrategy: .noRetry)) { _ in
            noIPv4Expectation.fulfill()
        }
        await fulfillment(of: [noIPv4Expectation], timeout: .UnitTest.timeout)
    }

    // func testSuccessGettingIPV6() async throws {
    //     let outgoingConnectionProxy = OutgoingConnectionProxy(
    //         apiContext: try makeApiContext(disableTls: false)
    //     )

    //     _ = try await outgoingConnectionProxy.getIPV6(retryStrategy: .noRetry)
    // }

    func testFailureGettingIPV6() async throws {
        let noIPv6Expectation = expectation(description: "Did not receive IPv6")

        let outgoingConnectionProxy = OutgoingConnectionProxy(
            apiContext: try makeApiContext(disableTls: true),
            hostname: "localhost:1")

        await XCTAssertThrowsErrorAsync(try await outgoingConnectionProxy.getIPV6(retryStrategy: .noRetry)) { _ in
            noIPv6Expectation.fulfill()
        }
        await fulfillment(of: [noIPv6Expectation], timeout: .UnitTest.timeout)
    }
}

private func makeApiContext(disableTls: Bool) throws -> ApiContext {
    let shadowsocksLoader = ShadowsocksLoader(
        cache: ShadowsocksConfigurationCacheStub(),
        relaySelector: ShadowsocksRelaySelectorStub(relays: .mock()),
        tunnelSettings: LatestTunnelSettings(),
        settingsUpdater: SettingsUpdater(listener: TunnelSettingsListener())
    )

    let accessMethodsRepository = AccessMethodRepositoryStub.stub

    return ApiContext(
        host: "localhost",
        address: REST.defaultAPIEndpoint.description,
        domain: REST.encryptedDNSHostname,
        disableTls: disableTls,
        shadowsocksProvider: shadowsocksLoader,
        accessMethodWrapper: initAccessMethodSettingsWrapper(methods: accessMethodsRepository.fetchAll()),
        accessMethodChangeListeners: []
    )
}
