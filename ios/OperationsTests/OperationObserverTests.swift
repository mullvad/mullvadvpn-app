//
//  OperationObserverTests.swift
//  MullvadVPNTests
//
//  Created by pronebird on 02/06/2022.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadMockData
import Operations
import XCTest

class OperationObserverTests: XCTestCase {
    func testBlockObserver() throws {
        let expectDidAttach = expectation(description: "didAttach handler")
        let expectDidStart = expectation(description: "didStart handler")
        let expectDidCancel = expectation(description: "didCancel handler")
        expectDidCancel.isInverted = true
        let expectDidFinish = expectation(description: "didAttach handler")

        let operation = AsyncBlockOperation {}
        operation.addBlockObserver(OperationBlockObserver(
            didAttach: { _ in
                expectDidAttach.fulfill()
            }, didStart: { _ in
                expectDidStart.fulfill()
            }, didCancel: { _ in
                expectDidCancel.fulfill()
            }, didFinish: { _, _ in
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
        operation.addBlockObserver(OperationBlockObserver(
            didAttach: { _ in
                expectDidAttach.fulfill()
            }, didStart: { _ in
                expectDidStart.fulfill()
            }, didCancel: { _ in
                expectDidCancel.fulfill()
            }, didFinish: { _, _ in
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
