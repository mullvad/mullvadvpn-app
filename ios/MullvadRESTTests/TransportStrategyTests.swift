//
//  TransportStrategyTests.swift
//  MullvadRESTTests
//
//  Created by Marco Nikic on 2023-04-27.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadREST
@testable import MullvadTypes
import XCTest

final class TransportStrategyTests: XCTestCase {
    var userDefaults: UserDefaults!
    static var suiteName: String!

    override class func setUp() {
        super.setUp()
        suiteName = UUID().uuidString
    }

    override func setUpWithError() throws {
        try super.setUpWithError()
        userDefaults = UserDefaults(suiteName: Self.suiteName)
    }

    override func tearDownWithError() throws {
        userDefaults.removePersistentDomain(forName: Self.suiteName)
        try super.tearDownWithError()
    }

    func testEveryThirdConnectionAttemptsIsDirect() {
        loopStrategyTest(with: TransportStrategy(userDefaults), in: 0 ... 12)
    }

    func testOverflowingConnectionAttempts() {
        userDefaults.set(Int.max, forKey: TransportStrategy.connectionAttemptsSharedCacheKey)
        let strategy = TransportStrategy(userDefaults)

        // (Int.max - 1) is a multiple of 3, so skip the first iteration
        loopStrategyTest(with: strategy, in: 1 ... 12)
    }

    func testConnectionAttemptsAreRecordedAfterFailure() {
        var strategy = TransportStrategy(userDefaults)

        strategy.didFail()

        let recordedValue = userDefaults.integer(forKey: TransportStrategy.connectionAttemptsSharedCacheKey)
        XCTAssertEqual(1, recordedValue)
    }

    private func loopStrategyTest(with strategy: TransportStrategy, in range: ClosedRange<Int>) {
        var strategy = strategy

        for index in range {
            let expectedResult: TransportStrategy.Transport
            expectedResult = index.isMultiple(of: 3) ? .useURLSession : .useShadowsocks
            XCTAssertEqual(strategy.connectionTransport(), expectedResult)
            strategy.didFail()
        }
    }
}
