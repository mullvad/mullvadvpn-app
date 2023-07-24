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
    func testEveryThirdConnectionAttemptsIsDirect() {
        loopStrategyTest(with: TransportStrategy(attemptsRecorder: MockRecorder()), in: 0 ... 12)
    }

    func testOverflowingConnectionAttempts() {
        let strategy = TransportStrategy(connectionAttempts: Int.max, attemptsRecorder: MockRecorder())

        // (Int.max - 1) is a multiple of 3, so skip the first iteration
        loopStrategyTest(with: strategy, in: 1 ... 12)
    }

    func testConnectionAttemptsAreRecordedAfterFailure() {
        var recorder = MockRecorder()
        var strategy = TransportStrategy(attemptsRecorder: recorder)

        recorder.didRecord = { connectionAttempt in
            XCTAssertEqual(connectionAttempt, 1)
        }

        strategy.didFail()
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

struct MockRecorder: AttemptsRecording {
    var didRecord: ((Int) -> Void)?

    func record(_ attempts: Int) {
        didRecord?(attempts)
    }
}
