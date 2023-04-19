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
            let expectedResult: REST.TransportStrategy.Transport
            expectedResult = index.isMultiple(of: 3) ? .useURLSession : .useShadowSocks
            XCTAssertEqual(strategy.connectionTransport(), expectedResult)
            strategy.didFail()
        }
    }

    func testLoadingFromCacheDoesNotImpactStrategy() throws {
        var strategy = REST.TransportStrategy()

        // Fail twice, the next suggested transport mode should be via Shadowsocks proxy
        strategy.didFail()
        strategy.didFail()
        XCTAssertEqual(strategy.connectionTransport(), .useShadowSocks)

        // Serialize the strategy and reload it from memory to simulate an application restart
        let encodedRawStrategy = try JSONEncoder().encode(strategy)
        var reloadedStrategy = try JSONDecoder().decode(REST.TransportStrategy.self, from: encodedRawStrategy)

        // This should be the third failure, the next suggested transport will be a direct one
        reloadedStrategy.didFail()
        XCTAssertEqual(reloadedStrategy.connectionTransport(), .useURLSession)
    }
}
