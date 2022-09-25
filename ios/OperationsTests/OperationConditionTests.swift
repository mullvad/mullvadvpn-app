//
//  OperationConditionTests.swift
//  MullvadVPNTests
//
//  Created by pronebird on 02/06/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Operations
import XCTest

class OperationConditionTests: XCTestCase {
    func testTrueCondition() {
        let expectConditionEvaluation = expectation(description: "Expect condition evaluation")
        let expectOperationToExecute = expectation(description: "Expect operation to execute")

        let operation = AsyncBlockOperation {
            expectOperationToExecute.fulfill()
        }

        let blockCondition = BlockCondition { op, completion in
            expectConditionEvaluation.fulfill()
            completion(true)
        }

        operation.addCondition(blockCondition)

        let operationQueue = AsyncOperationQueue()
        operationQueue.addOperation(operation)

        waitForExpectations(timeout: 1)
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

        let blockCondition = BlockCondition { op, completion in
            expectConditionEvaluation.fulfill()
            completion(false)
        }

        operation.addCondition(blockCondition)

        let operationQueue = AsyncOperationQueue()
        operationQueue.addOperation(operation)

        waitForExpectations(timeout: 1)
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

        waitForExpectations(timeout: 1)
    }

    func testNoFailedDependenciesCondition() {
        let expectToNeverExecute = expectation(description: "Expect child to never execute.")
        expectToNeverExecute.isInverted = true

        let parent = ResultBlockOperation<Void, URLError> {
            throw URLError(.badURL)
        }

        let child = AsyncBlockOperation {
            expectToNeverExecute.fulfill()
        }
        child.addDependency(parent)
        child.addCondition(NoFailedDependenciesCondition(ignoreCancellations: false))

        let operationQueue = AsyncOperationQueue()
        operationQueue.addOperations([parent, child], waitUntilFinished: false)

        waitForExpectations(timeout: 1)
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

        waitForExpectations(timeout: 1)
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

        let firstOperation = AsyncBlockOperation { op in
            DispatchQueue.main.asyncAfter(deadline: .now() + .seconds(1)) {
                expectFirstOperationExecution.fulfill()
                op.finish()
            }
        }
        firstOperation.addCondition(MutuallyExclusive(category: exclusiveCategory))

        let secondOperation = AsyncBlockOperation {
            expectSecondOperationExecution.fulfill()
        }
        secondOperation.addCondition(MutuallyExclusive(category: exclusiveCategory))

        operationQueue.addOperations([firstOperation, secondOperation], waitUntilFinished: false)

        let expectations = [expectFirstOperationExecution, expectSecondOperationExecution]
        wait(for: expectations, timeout: 2, enforceOrder: true)
    }
}
