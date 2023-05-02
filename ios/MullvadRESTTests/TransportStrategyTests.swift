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

        // Fail twice, the next suggested connection mode should be via Shadowsocks proxy
        strategy.didFail()
        strategy.didFail()
        assertStrategy(.useShadowSocks, actual: strategy.connectionTransport())

        // Serialize the strategy and reload it from memory to simulate an application restart
        let encodedRawStrategy = try JSONEncoder().encode(strategy)
        var reloadedStrategy = try JSONDecoder().decode(REST.TransportStrategy.self, from: encodedRawStrategy)

        // This should be the third failure, the next suggested strategy falls back to trying a direct connection
        reloadedStrategy.didFail()
        assertStrategy(.useURLSession, actual: reloadedStrategy.connectionTransport())
    }

    func testTimingOutForcesADifferentTransport() {
        var strategy = REST.TransportStrategy()

        // Fail once, forcing the next strategy to be Shadowsocks
        strategy.didFail()
        assertStrategy(.useShadowSocks, actual: strategy.connectionTransport())

        // Fail with a timeout, forcing the strategy to go back to attempting a direct connection
        strategy.didFail(code: .timedOut)
        assertStrategy(.useURLSession, actual: strategy.connectionTransport())
    }

    func testTimingOutAlwaysForcesDifferentTransport() {
        var strategy = REST.TransportStrategy()

        // First timeout should force to transport to shadowSocks
        strategy.didFail(code: .timedOut)
        assertStrategy(.useShadowSocks, actual: strategy.connectionTransport())

        // Second timeout should force attempting a direct connection again
        strategy.didFail(code: .timedOut)
        assertStrategy(.useURLSession, actual: strategy.connectionTransport())

        // Third fail should force shadow socks again
        strategy.didFail(code: .timedOut)
        assertStrategy(.useShadowSocks, actual: strategy.connectionTransport())
    }

    func testFailingAfterTimeoutDoesNotAffectStrategy() {
        var strategy = REST.TransportStrategy()

        // Fail twice, but the second failure is a timeout, switching back to direct transport
        strategy.didFail()
        strategy.didFail(code: .timedOut)
        assertStrategy(.useURLSession, actual: strategy.connectionTransport())

        // The next two failure should force Shadowsocks transports
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
