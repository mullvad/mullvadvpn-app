//
//  AsyncBlockOperationTests.swift
//  OperationsTests
//
//  Created by pronebird on 26/04/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadMockData
import MullvadTypes
import Operations
import XCTest

final class AsyncBlockOperationTests: XCTestCase {
    let operationQueue = AsyncOperationQueue()

    func testBlockOperation() async {
        let executionExpectation = expectation(description: "Should execute")
        let finishExpectation = expectation(description: "Should finish")

        let operation = AsyncBlockOperation(block: { finish in
            executionExpectation.fulfill()
            finish(nil)
        })

        operation.completionBlock = {
            finishExpectation.fulfill()
        }

        operationQueue.addOperation(operation)

        await fulfillment(of: [executionExpectation, finishExpectation], timeout: .UnitTest.timeout)
    }

    func testSynchronousBlockOperation() async {
        let executionExpectation = expectation(description: "Should execute")
        let finishExpectation = expectation(description: "Should finish")

        let operation = AsyncBlockOperation {
            executionExpectation.fulfill()
        }

        operation.completionBlock = {
            finishExpectation.fulfill()
        }

        operationQueue.addOperation(operation)

        await fulfillment(of: [executionExpectation, finishExpectation], timeout: .UnitTest.timeout)
    }

    func testCancellableTaskBlockOperation() async {
        let executionExpectation = expectation(description: "Should execute")
        let cancelExpectation = expectation(description: "Should cancel")
        let finishExpectation = expectation(description: "Should finish")

        let operation = AsyncBlockOperation { finish -> Cancellable in
            executionExpectation.fulfill()

            return AnyCancellable {
                cancelExpectation.fulfill()
                finish(nil)
            }
        }

        operation.completionBlock = {
            finishExpectation.fulfill()
        }

        operation.onStart { op in
            op.cancel()
        }

        operationQueue.addOperation(operation)

        await fulfillment(of: [executionExpectation, cancelExpectation, finishExpectation], timeout: .UnitTest.timeout)
    }

    func testCancellationShouldNotFireBeforeOperationIsEnqueued() async throws {
        let expect = expectation(description: "Cancellation should not fire.")
        expect.isInverted = true

        let operation = AsyncBlockOperation {}
        operation.onCancel { _ in expect.fulfill() }
        operation.cancel()

        await fulfillment(of: [expect], timeout: .UnitTest.invertedTimeout)
    }

    func testCancellationShouldFireAfterCancelledOperationIsEnqueued() async throws {
        let expect = expectation(description: "Cancellation should fire.")

        let operation = AsyncBlockOperation {}
        operation.onCancel { _ in expect.fulfill() }
        operation.cancel()
        operationQueue.addOperation(operation)

        await fulfillment(of: [expect], timeout: .UnitTest.timeout)
    }
}
