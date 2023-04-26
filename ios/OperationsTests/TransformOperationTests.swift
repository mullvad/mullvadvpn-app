//
//  TransformOperationTests.swift
//  OperationsTests
//
//  Created by pronebird on 26/04/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
import Operations
import XCTest

final class TransformOperationTests: XCTestCase {
    let operationQueue = AsyncOperationQueue()

    func testBlockTransformOperation() {
        let finishExpectation = expectation(description: "Should finish")

        let transform = TransformOperation(input: Int.zero) { input, op in
            op.finish(result: .success(input + 1))
        }

        transform.onFinish { op, error in
            XCTAssertEqual(op.result?.value, 1)

            finishExpectation.fulfill()
        }

        operationQueue.addOperation(transform)

        waitForExpectations(timeout: 1)
    }

    func testThrowingBlockTransformOperation() {
        let finishExpectation = expectation(description: "Should finish")

        let transform = TransformOperation(input: Int.zero) { value in
            throw URLError(.badURL)
        }

        transform.onFinish { op, error in
            XCTAssertEqual(error as? URLError, URLError(.badURL))

            finishExpectation.fulfill()
        }

        operationQueue.addOperation(transform)

        waitForExpectations(timeout: 1)
    }

    func testCancellableTaskBlockTranasformOperation() {
        let finishExpectation = expectation(description: "Should finish")

        let transform = TransformOperation<Int, Int>(input: Int.zero, cancellableTask: { _, op in
            return AnyCancellable {
                op.finish(result: .failure(URLError(.cancelled)))
            }
        })

        transform.onStart { op in
            op.cancel()
        }

        transform.onFinish { op, error in
            XCTAssertEqual(error as? URLError, URLError(.cancelled))

            finishExpectation.fulfill()
        }

        operationQueue.addOperation(transform)

        waitForExpectations(timeout: 1)
    }

    func testShouldFailWithUnsatisfiedRequirement() {
        let finishExpectation = expectation(description: "Should finish")

        let transform = TransformOperation<Int, Int> { input, op in
            op.finish(result: .success(input))
        }

        transform.onFinish { _, error in
            XCTAssertEqual(error as? OperationError, .unsatisfiedRequirement)

            finishExpectation.fulfill()
        }

        operationQueue.addOperation(transform)

        waitForExpectations(timeout: 1)
    }
}
