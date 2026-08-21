// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import XCTest

@testable import MullvadREST
@testable import MullvadTypes

class RetryStrategyTests: XCTestCase {
    func testJitteredBackoffDoesNotGoBeyondMaxDelay() throws {
        let maxDelay = REST.CodableDuration(seconds: 10, attoseconds: 0)
        let retryDelay = REST.RetryDelay.exponentialBackoff(initial: .seconds(1), multiplier: 2, maxDelay: maxDelay)
        let retry = REST.RetryStrategy(maxRetryCount: 0, delay: retryDelay, applyJitter: true)
        let iterator = retry.makeDelayIterator()
        var previousDelay = Duration(secondsComponent: 0, attosecondsComponent: 0)

        for _ in 0...10 {
            let currentDelay = try XCTUnwrap(iterator.next())
            XCTAssertLessThanOrEqual(previousDelay, currentDelay)
            XCTAssertLessThanOrEqual(currentDelay, maxDelay.duration)
            previousDelay = currentDelay
        }
    }

    func testJitteredConstantCannotBeMoreThanDouble() throws {
        let retryDelay = REST.RetryDelay.constant(.seconds(10))
        let retry = REST.RetryStrategy(maxRetryCount: 0, delay: retryDelay, applyJitter: true)
        let iterator = retry.makeDelayIterator()
        let minimumDelay = Duration(secondsComponent: 10, attosecondsComponent: 0)
        let maximumDelay = Duration(secondsComponent: 20, attosecondsComponent: 0)

        for _ in 0...10 {
            let currentDelay = try XCTUnwrap(iterator.next())
            let maximumJitterRange = minimumDelay...maximumDelay
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
