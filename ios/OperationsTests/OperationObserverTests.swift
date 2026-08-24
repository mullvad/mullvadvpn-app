// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Operations
import XCTest

@testable import MullvadMockData

class OperationObserverTests: XCTestCase {
    func testBlockObserver() throws {
        let expectDidAttach = expectation(description: "didAttach handler")
        let expectDidStart = expectation(description: "didStart handler")
        let expectDidCancel = expectation(description: "didCancel handler")
        expectDidCancel.isInverted = true
        let expectDidFinish = expectation(description: "didAttach handler")

        let operation = AsyncBlockOperation {}
        operation.addBlockObserver(
            OperationBlockObserver(
                didAttach: { _ in
                    expectDidAttach.fulfill()
                },
                didStart: { _ in
                    expectDidStart.fulfill()
                },
                didCancel: { _ in
                    expectDidCancel.fulfill()
                },
                didFinish: { _, _ in
                    expectDidFinish.fulfill()
                }
            ))

        let operationQueue = AsyncOperationQueue()
        operationQueue.addOperation(operation)

        wait(for: [expectDidCancel], timeout: .UnitTest.invertedTimeout)
        wait(for: [expectDidAttach, expectDidStart, expectDidFinish], timeout: .UnitTest.timeout, enforceOrder: true)
    }

    func testBlockObserverWithCancelledOperation() {
        let expectDidAttach = expectation(description: "didAttach handler")
        let expectDidStart = expectation(description: "didStart handler")
        expectDidStart.isInverted = true
        let expectDidCancel = expectation(description: "didCancel handler")
        let expectDidFinish = expectation(description: "didAttach handler")

        let operation = AsyncBlockOperation {}
        operation.addBlockObserver(
            OperationBlockObserver(
                didAttach: { _ in
                    expectDidAttach.fulfill()
                },
                didStart: { _ in
                    expectDidStart.fulfill()
                },
                didCancel: { _ in
                    expectDidCancel.fulfill()
                },
                didFinish: { _, _ in
                    expectDidFinish.fulfill()
                }
            ))
        operation.cancel()

        let operationQueue = AsyncOperationQueue()
        operationQueue.addOperation(operation)

        wait(for: [expectDidStart], timeout: .UnitTest.invertedTimeout)
        wait(for: [expectDidAttach, expectDidCancel, expectDidFinish], timeout: .UnitTest.timeout, enforceOrder: true)
    }
}
