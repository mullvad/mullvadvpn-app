//
//  AsyncResultBlockOperationTests.swift
//  OperationsTests
//
//  Created by pronebird on 26/04/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadMockData
import MullvadTypes
import Operations
import XCTest

@MainActor
final class AsyncResultBlockOperationTests: XCTestCase {
    let operationQueue = AsyncOperationQueue()

    func testBlockOperation() {
        let expectation = expectation(description: "Should finish")

        let operation = ResultBlockOperation<Bool> { finish in
            finish(.success(true))
        }

        operation.onFinish { op, _ in
            XCTAssertEqual(op.result?.value, true)
            expectation.fulfill()
        }

        operationQueue.addOperation(operation)

        waitForExpectations(timeout: .UnitTest.timeout)
    }

    func testThrowingBlockOperation() {
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

        waitForExpectations(timeout: .UnitTest.timeout)
    }

    func testCancellableTaskOperation() {
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

        waitForExpectations(timeout: .UnitTest.timeout)
    }
}
