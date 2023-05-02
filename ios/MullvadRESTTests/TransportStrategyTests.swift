//
//  TransportStrategyTests.swift
//  MullvadRESTTests
//
//  Created by Marco Nikic on 2023-04-27.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import XCTest

final class TransportStrategyTests: XCTestCase {
    func testEveryThirdConnectionAttemptsIsDirect() {
        var strategy = REST.TransportStrategy()

        for index in 0 ... 12 {
            let expectedResult: REST.TransportStrategy.Transport = index % 3 == 0 ? .useURLSession : .useShadowSocks
            XCTAssertEqual(expectedResult, strategy.connectionTransport())
            strategy.didFail()
        }
    }

    func testDefaultConnectionTransportIsDirectURLSession() {
        let strategy = REST.TransportStrategy()
        assertStrategy(.useURLSession, actual: strategy.connectionTransport())
    }

    func testLoadingFromCacheDoesNotImpactStrategy() throws {
        var strategy = REST.TransportStrategy()

        // Fail twice, the next suggested transport mode should be via Shadowsocks proxy
        strategy.didFail()
        strategy.didFail()
        assertStrategy(.useShadowSocks, actual: strategy.connectionTransport())

        // Serialize the strategy and reload it from memory to simulate an application restart
        let encodedRawStrategy = try JSONEncoder().encode(strategy)
        var reloadedStrategy = try JSONDecoder().decode(REST.TransportStrategy.self, from: encodedRawStrategy)

        // This should be the third failure, the next suggested transport will be a direct one
        reloadedStrategy.didFail()
        assertStrategy(.useURLSession, actual: reloadedStrategy.connectionTransport())
    }

    func testTimingOutAlwaysSuggestsDifferentTransport() {
        var strategy = REST.TransportStrategy()

        // First timeout should suggest a ShadowSocks transport
        strategy.didFail(code: .timedOut)
        assertStrategy(.useShadowSocks, actual: strategy.connectionTransport())

        // Second timeout should suggest a direct transport
        strategy.didFail(code: .timedOut)
        assertStrategy(.useURLSession, actual: strategy.connectionTransport())

        // Third timeout should suggest a Shadowsocks transport again
        strategy.didFail(code: .timedOut)
        assertStrategy(.useShadowSocks, actual: strategy.connectionTransport())
    }

    func testFailingAfterTimeoutDoesNotAffectStrategy() {
        var strategy = REST.TransportStrategy()

        // Fail twice, but the second failure is a timeout, switching back to direct transport
        strategy.didFail()
        strategy.didFail(code: .timedOut)
        assertStrategy(.useURLSession, actual: strategy.connectionTransport())

        // The next two failures should suggest Shadowsocks transports
        strategy.didFail()
        assertStrategy(.useShadowSocks, actual: strategy.connectionTransport())
        strategy.didFail()
        assertStrategy(.useShadowSocks, actual: strategy.connectionTransport())
    }
}

extension TransportStrategyTests {
    func assertStrategy(_ expected: REST.TransportStrategy.Transport, actual: REST.TransportStrategy.Transport) {
        XCTAssertEqual(expected, actual)
    }
}
