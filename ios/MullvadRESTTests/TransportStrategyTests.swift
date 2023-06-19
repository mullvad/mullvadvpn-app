//
//  TransportStrategyTests.swift
//  MullvadRESTTests
//
//  Created by Marco Nikic on 2023-04-27.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadREST
import XCTest

final class TransportStrategyTests: XCTestCase {
    func testEveryThirdConnectionAttemptsIsDirect() {
        loopStrategyTest(with: TransportStrategy())
    }

    func testOverflowingConnectionAttempts() {
        var strategy = TransportStrategy()
        strategy.connectionAttempts = UInt.max

        loopStrategyTest(with: strategy)
    }

    func testLoadingFromCacheDoesNotImpactStrategy() throws {
        var strategy = TransportStrategy()

        // Fail twice, the next suggested transport mode should be via Shadowsocks proxy
        strategy.didFail()
        strategy.didFail()
        XCTAssertEqual(strategy.connectionTransport(), .useShadowsocks)

        // Serialize the strategy and reload it from memory to simulate an application restart
        let encodedRawStrategy = try JSONEncoder().encode(strategy)
        var reloadedStrategy = try JSONDecoder().decode(TransportStrategy.self, from: encodedRawStrategy)

        // This should be the third failure, the next suggested transport will be a direct one
        reloadedStrategy.didFail()
        XCTAssertEqual(reloadedStrategy.connectionTransport(), .useURLSession)
    }

    private func loopStrategyTest(with strategy: TransportStrategy) {
        var strategy = strategy

        for index in 0 ... 12 {
            let expectedResult: TransportStrategy.Transport
            expectedResult = index.isMultiple(of: 3) ? .useURLSession : .useShadowsocks
            XCTAssertEqual(strategy.connectionTransport(), expectedResult)
            strategy.didFail()
        }
    }
}
