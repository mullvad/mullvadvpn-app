// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadTypes
import Operations
import XCTest

@testable import MullvadMockData

final class AsyncResultBlockOperationTests: XCTestCase {
    let operationQueue = AsyncOperationQueue()

    func testBlockOperation() async {
        let expectation = expectation(description: "Should finish")

        let operation = ResultBlockOperation<Bool> { finish in
            finish(.success(true))
        }

        operation.onFinish { op, _ in
            XCTAssertEqual(op.result?.value, true)
            expectation.fulfill()
        }

        operationQueue.addOperation(operation)

        await fulfillment(of: [expectation], timeout: .UnitTest.timeout)
    }

    func testThrowingBlockOperation() async {
        let expectation = expectation(description: "Should finish")

        let operation = ResultBlockOperation {
            throw URLError(.badURL)
        }

        operation.onFinish { op, error in
            XCTAssertEqual(op.result?.error as? URLError, URLError(.badURL))
            XCTAssertEqual(error as? URLError, URLError(.badURL))

            expectation.fulfill()
        }

        operationQueue.addOperation(operation)

        await fulfillment(of: [expectation], timeout: .UnitTest.timeout)
    }

    func testCancellableTaskOperation() async {
        let expectation = expectation(description: "Should finish")

        let operation = ResultBlockOperation<Bool> { finish -> Cancellable in
            AnyCancellable {
                finish(.failure(URLError(.cancelled)))
            }
        }

        operation.onStart { op in
            op.cancel()
        }

        operation.onFinish { op, error in
            XCTAssertEqual(op.result?.error as? URLError, URLError(.cancelled))
            XCTAssertEqual(error as? URLError, URLError(.cancelled))
            expectation.fulfill()
        }

        operationQueue.addOperation(operation)

        await fulfillment(of: [expectation], timeout: .UnitTest.timeout)
    }
}
