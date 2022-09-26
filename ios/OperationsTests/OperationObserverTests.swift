//
//  OperationObserverTests.swift
//  MullvadVPNTests
//
//  Created by pronebird on 02/06/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Operations
import XCTest

class OperationObserverTests: XCTestCase {
    func testBlockObserver() throws {
        let expectDidAttach = expectation(description: "didAttach handler")
        let expectDidStart = expectation(description: "didStart handler")
        let expectDidCancel = expectation(description: "didCancel handler")
        expectDidCancel.isInverted = true
        let expectDidFinish = expectation(description: "didAttach handler")

        let operation = AsyncBlockOperation()
        operation.addBlockObserver(OperationBlockObserver(
            didAttach: { op in
                expectDidAttach.fulfill()
            }, didStart: { op in
                expectDidStart.fulfill()
            }, didCancel: { op in
                expectDidCancel.fulfill()
            }, didFinish: { op, error in
                expectDidFinish.fulfill()
            }
        ))

        let operationQueue = AsyncOperationQueue()
        operationQueue.addOperation(operation)

        let expectations = [expectDidCancel, expectDidAttach, expectDidStart, expectDidFinish]
        wait(for: expectations, timeout: 1, enforceOrder: true)
    }

    func testBlockObserverWithCancelledOperation() {
        let expectDidAttach = expectation(description: "didAttach handler")
        let expectDidStart = expectation(description: "didStart handler")
        expectDidStart.isInverted = true
        let expectDidCancel = expectation(description: "didCancel handler")
        let expectDidFinish = expectation(description: "didAttach handler")

        let operation = AsyncBlockOperation()
        operation.addBlockObserver(OperationBlockObserver(
            didAttach: { op in
                expectDidAttach.fulfill()
            }, didStart: { op in
                expectDidStart.fulfill()
            }, didCancel: { op in
                expectDidCancel.fulfill()
            }, didFinish: { op, error in
                expectDidFinish.fulfill()
            }
        ))
        operation.cancel()

        let operationQueue = AsyncOperationQueue()
        operationQueue.addOperation(operation)

        let expectations = [expectDidAttach, expectDidCancel, expectDidStart, expectDidFinish]
        wait(for: expectations, timeout: 1, enforceOrder: true)
    }
}
