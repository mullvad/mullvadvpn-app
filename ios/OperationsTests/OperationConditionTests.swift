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

@MainActor
class OperationConditionTests: XCTestCase {
    func testTrueCondition() {
        let expectConditionEvaluation = expectation(description: "Expect condition evaluation")
        let expectOperationToExecute = expectation(description: "Expect operation to execute")

        let operation = AsyncBlockOperation {
            expectOperationToExecute.fulfill()
        }

        let blockCondition = BlockCondition { _, completion in
            expectConditionEvaluation.fulfill()
            completion(true)
        }

        operation.addCondition(blockCondition)

        let operationQueue = AsyncOperationQueue()
        operationQueue.addOperation(operation)

        waitForExpectations(timeout: .UnitTest.timeout)
    }

    func testFalseCondition() {
        let expectConditionEvaluation = expectation(description: "Expect condition evaluation")
        let expectOperationToNeverExecute = expectation(
            description: "Expect operation to never execute"
        )
        expectOperationToNeverExecute.isInverted = true

        let operation = AsyncBlockOperation {
            expectOperationToNeverExecute.fulfill()
        }

        let blockCondition = BlockCondition { _, completion in
            expectConditionEvaluation.fulfill()
            completion(false)
        }

        operation.addCondition(blockCondition)

        let operationQueue = AsyncOperationQueue()
        operationQueue.addOperation(operation)

        wait(for: [expectOperationToNeverExecute], timeout: .UnitTest.invertedTimeout)
        wait(for: [expectConditionEvaluation], timeout: .UnitTest.timeout)
    }

    func testNoCancelledDependenciesCondition() {
        let expectToNeverExecute = expectation(description: "Expect child to never execute.")
        expectToNeverExecute.isInverted = true

        let parent = BlockOperation()
        parent.cancel()

        let child = AsyncBlockOperation {
            expectToNeverExecute.fulfill()
        }
        child.addDependency(parent)
        child.addCondition(NoCancelledDependenciesCondition())

        let operationQueue = AsyncOperationQueue()
        operationQueue.addOperations([parent, child], waitUntilFinished: false)

        waitForExpectations(timeout: .UnitTest.invertedTimeout)
    }

    func testNoFailedDependenciesCondition() {
        let expectToNeverExecute = expectation(description: "Expect child to never execute.")
        expectToNeverExecute.isInverted = true

        let parent = ResultBlockOperation<Void> {
            throw URLError(.badURL)
        }

        let child = AsyncBlockOperation {
            expectToNeverExecute.fulfill()
        }
        child.addDependency(parent)
        child.addCondition(NoFailedDependenciesCondition(ignoreCancellations: false))

        let operationQueue = AsyncOperationQueue()
        operationQueue.addOperations([parent, child], waitUntilFinished: false)

        waitForExpectations(timeout: .UnitTest.invertedTimeout)
    }

    func testNoFailedDependenciesIgnoringCancellationsCondition() {
        let expectToExecute = expectation(description: "Expect child to execute.")

        let parent = BlockOperation()
        parent.cancel()

        let child = AsyncBlockOperation {
            expectToExecute.fulfill()
        }
        child.addDependency(parent)
        child.addCondition(NoFailedDependenciesCondition(ignoreCancellations: true))

        let operationQueue = AsyncOperationQueue()
        operationQueue.addOperations([parent, child], waitUntilFinished: false)

        waitForExpectations(timeout: .UnitTest.timeout)
    }

    func testMutuallyExclusiveCondition() {
        let expectFirstOperationExecution = expectation(
            description: "Expect first operation to execute first"
        )
        let expectSecondOperationExecution = expectation(
            description: "Expect second operation to execute last"
        )

        let exclusiveCategory = "exclusiveOperations"
        let operationQueue = AsyncOperationQueue()

        let firstOperation = AsyncBlockOperation { finish in
            DispatchQueue.main.asyncAfter(deadline: .now() + .seconds(1)) {
                expectFirstOperationExecution.fulfill()
                finish(nil)
            }
        }
        firstOperation.addCondition(MutuallyExclusive(category: exclusiveCategory))

        let secondOperation = AsyncBlockOperation {
            expectSecondOperationExecution.fulfill()
        }
        secondOperation.addCondition(MutuallyExclusive(category: exclusiveCategory))

        operationQueue.addOperations([firstOperation, secondOperation], waitUntilFinished: false)

        let expectations = [expectFirstOperationExecution, expectSecondOperationExecution]
        wait(for: expectations, timeout: .UnitTest.timeout, enforceOrder: true)
    }
}
