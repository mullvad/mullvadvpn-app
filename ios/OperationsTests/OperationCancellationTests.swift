//
//  OperationCancellationTests.swift
//  OperationsTests
//
//  Created by pronebird on 12/04/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Operations
import XCTest

final class OperationCancellationTests: XCTestCase {
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

        let operationQueue = AsyncOperationQueue()
        let operation = AsyncBlockOperation {}
        operation.onCancel { _ in expect.fulfill() }
        operation.cancel()
        operationQueue.addOperation(operation)

        waitForExpectations(timeout: 1)
    }
}
