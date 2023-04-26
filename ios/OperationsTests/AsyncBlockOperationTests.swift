//
//  AsyncBlockOperationTests.swift
//  OperationsTests
//
//  Created by pronebird on 26/04/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
import Operations
import XCTest

final class AsyncBlockOperationTests: XCTestCase {
    let operationQueue = AsyncOperationQueue()

    func testBlockOperation() {
        let executionExpectation = expectation(description: "Should execute")
        let finishExpectation = expectation(description: "Should finish")

        let operation = AsyncBlockOperation(block: { op in
            executionExpectation.fulfill()
            op.finish()
        })

        operation.completionBlock = {
            finishExpectation.fulfill()
        }

        operationQueue.addOperation(operation)

        waitForExpectations(timeout: 1)
    }

    func testSynchronousBlockOperation() {
        let executionExpectation = expectation(description: "Should execute")
        let finishExpectation = expectation(description: "Should finish")

        let operation = AsyncBlockOperation {
            executionExpectation.fulfill()
        }

        operation.completionBlock = {
            finishExpectation.fulfill()
        }

        operationQueue.addOperation(operation)

        waitForExpectations(timeout: 1)
    }

    func testCancellableTaskBlockOperation() {
        let executionExpectation = expectation(description: "Should execute")
        let cancelExpectation = expectation(description: "Should cancel")
        let finishExpectation = expectation(description: "Should finish")

        let operation = AsyncBlockOperation(cancellableTask: { op in
            executionExpectation.fulfill()

            return AnyCancellable {
                cancelExpectation.fulfill()

                op.finish()
            }
        })

        operation.completionBlock = {
            finishExpectation.fulfill()
        }

        operation.onStart { op in
            op.cancel()
        }

        operationQueue.addOperation(operation)

        waitForExpectations(timeout: 100)
    }

    func testCancellationShouldNotFireBeforeOperationIsEnqueued() throws {
        let expect = expectation(description: "Cancellation should not fire.")
        expect.isInverted = true

        let operation = AsyncBlockOperation {}
        operation.onCancel { _ in expect.fulfill() }
        operation.cancel()

        waitForExpectations(timeout: 1)
    }

    func testCancellationShouldFireAfterCancelledOperationIsEnqueued() throws {
        let expect = expectation(description: "Cancellation should fire.")

        let operation = AsyncBlockOperation {}
        operation.onCancel { _ in expect.fulfill() }
        operation.cancel()
        operationQueue.addOperation(operation)

        waitForExpectations(timeout: 1)
    }
}
