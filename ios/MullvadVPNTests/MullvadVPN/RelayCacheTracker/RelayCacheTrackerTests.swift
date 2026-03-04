//
//  RelayCacheTrackerTests.swift
//  MullvadVPNTests
//
//  Created on 2026-03-03.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadMockData
import XCTest

@testable import MullvadMockData
@testable import MullvadREST
@testable import MullvadTypes

class RelayCacheTrackerTests: XCTestCase {
    func testUpdateRelaysIsThrottledWhenCacheIsFresh() {
        let tracker = makeTracker()

        let expectation = expectation(description: "Completion called")
        _ = tracker.updateRelays { result in
            guard case .success(.throttled) = result else {
                XCTFail("Expected .throttled, got \(result)")
                expectation.fulfill()
                return
            }
            expectation.fulfill()
        }

        wait(for: [expectation], timeout: 5)
    }

    func testFetchRelaysBypassesThrottle() {
        let tracker = makeTracker()

        let expectation = expectation(description: "Completion called")
        _ = tracker.fetchRelays { result in
            guard case .success(.sameContent) = result else {
                XCTFail("Expected .sameContent, got \(result)")
                expectation.fulfill()
                return
            }
            expectation.fulfill()
        }

        wait(for: [expectation], timeout: 5)
    }

    private func makeTracker() -> RelayCacheTracker {
        let apiProxy = APIProxyStub(getRelaysResult: .success(.notModified))
        return RelayCacheTracker(
            relayCache: MockRelayCache(),
            backgroundTaskProvider: UIApplicationStub(),
            apiProxy: apiProxy
        )
    }
}
