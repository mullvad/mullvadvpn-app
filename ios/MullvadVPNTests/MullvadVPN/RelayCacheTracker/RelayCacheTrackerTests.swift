// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
