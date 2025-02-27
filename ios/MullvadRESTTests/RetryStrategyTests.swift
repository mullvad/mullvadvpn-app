//
//  RetryStrategyTests.swift
//  MullvadRESTTests
//
//  Created by Marco Nikic on 2024-06-07.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
@testable import MullvadREST
@testable import MullvadTypes
import XCTest

class RetryStrategyTests: XCTestCase {
    func testJitteredBackoffDoesNotGoBeyondMaxDelay() throws {
        let maxDelay = REST.CodableDuration(seconds: 10, attoseconds: 0)
        let retryDelay = REST.RetryDelay.exponentialBackoff(initial: .seconds(1), multiplier: 2, maxDelay: maxDelay)
        let retry = REST.RetryStrategy(maxRetryCount: 0, delay: retryDelay, applyJitter: true)
        let iterator = retry.makeDelayIterator()
        var previousDelay = Duration(secondsComponent: 0, attosecondsComponent: 0)

        for _ in 0 ... 10 {
            let currentDelay = try XCTUnwrap(iterator.next())
            XCTAssertLessThanOrEqual(previousDelay, currentDelay)
            XCTAssertLessThanOrEqual(currentDelay, maxDelay.toDuration)
            previousDelay = currentDelay
        }
    }

    func testJitteredConstantCannotBeMoreThanDouble() throws {
        let retryDelay = REST.RetryDelay.constant(.seconds(10))
        let retry = REST.RetryStrategy(maxRetryCount: 0, delay: retryDelay, applyJitter: true)
        let iterator = retry.makeDelayIterator()
        let minimumDelay = Duration(secondsComponent: 10, attosecondsComponent: 0)
        let maximumDelay = Duration(secondsComponent: 20, attosecondsComponent: 0)

        for _ in 0 ... 10 {
            let currentDelay = try XCTUnwrap(iterator.next())
            let maximumJitterRange = minimumDelay ... maximumDelay
            print(currentDelay)
            XCTAssertLessThanOrEqual(maximumJitterRange.lowerBound, currentDelay)
            XCTAssertGreaterThanOrEqual(maximumJitterRange.upperBound, currentDelay)
        }
    }

    func testCannotApplyJitterToNeverRetry() throws {
        let retryDelay = REST.RetryDelay.never
        let retry = REST.RetryStrategy(maxRetryCount: 0, delay: retryDelay, applyJitter: true)
        let iterator = retry.makeDelayIterator()
        XCTAssertNil(iterator.next())
    }
}
