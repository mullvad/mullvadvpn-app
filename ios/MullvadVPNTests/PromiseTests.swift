//
//  PromiseTests.swift
//  PromiseTests
//
//  Created by pronebird on 22/08/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import XCTest

class PromiseTests: XCTestCase {

    override func setUpWithError() throws {
        // Put setup code here. This method is called before the invocation of each test method in the class.
    }

    override func tearDownWithError() throws {
        // Put teardown code here. This method is called after the invocation of each test method in the class.
    }

    func testObserveResolvedPromise() throws {
        let expect = expectation(description: "Wait for promise")

        Promise(value: 1)
            .observe { completion in
                XCTAssertEqual(completion, .finished(1))
                expect.fulfill()
            }

        wait(for: [expect], timeout: 1)
    }

    func testObservePromise() throws {
        let expect = expectation(description: "Wait for promise")
        Promise<Int> { resolver in
            resolver.resolve(value: 1)
        }
        .observe { completion in
            XCTAssertEqual(completion, .finished(1))
            expect.fulfill()
        }

        wait(for: [expect], timeout: 1)
    }

    func testReceiveOn() throws {
        let expect = expectation(description: "Wait for promise")
        let queue = DispatchQueue(label: "TestQueue")

        Promise(value: 1)
            .receive(on: queue)
            .observe { completion in
                dispatchPrecondition(condition: .onQueue(queue))
                expect.fulfill()
            }

        wait(for: [expect], timeout: 1)
    }

    func testReceiveOnCancellation() {
        let expect = expectation(description: "Wait for promise to complete")

        let promise = Promise(value: 1)
            .receive(on: .main)

        promise.observe { completion in
            XCTAssertEqual(completion, .cancelled)
            expect.fulfill()
        }

        promise.cancel()

        wait(for: [expect], timeout: 1)
    }

    func testDelay() throws {
        let expect = expectation(description: "Wait for promise")
        let queue = DispatchQueue(label: "TestQueue")

        let startDate = Date()
        Promise.deferred { () -> Int in
            let elapsed = startDate.timeIntervalSinceNow * -1000
            XCTAssertGreaterThanOrEqual(elapsed, 100)
            dispatchPrecondition(condition: .onQueue(queue))
            expect.fulfill()
            return 1
        }
        .delay(by: .milliseconds(100), timerType: .walltime, queue: queue)
        .observe { _ in }

        wait(for: [expect], timeout: 1)
    }

    func testDelayCancellation() throws {
        let expect = expectation(description: "Should never fulfill")
        expect.isInverted = true

        let promise = Promise.deferred { () -> Int in
            expect.fulfill()
            return 1
        }.delay(by: .milliseconds(100), timerType: .walltime)

        promise.observe { completion in
            XCTAssertEqual(completion, .cancelled)
        }

        promise.cancel()

        wait(for: [expect], timeout: 1)
    }

    func testScheduleOn() throws {
        let expect = expectation(description: "Wait for promise")
        let queue = DispatchQueue(label: "TestQueue")

        Promise<Int> { resolver in
            dispatchPrecondition(condition: .onQueue(queue))
            resolver.resolve(value: 1)
        }
        .schedule(on: queue)
        .observe { completion in
            expect.fulfill()
        }

        wait(for: [expect], timeout: 1)
    }

    func testBlockOn() throws {
        let expect1 = expectation(description: "Wait for promise")
        let expect2 = expectation(description: "Wait for queue to be unblocked")
        let queue = DispatchQueue(label: "TestQueue")

        Promise<Int> { resolver in
            DispatchQueue.main.async {
                resolver.resolve(value: 1)
            }
        }
        .block(on: queue)
        .observe { completion in
            dispatchPrecondition(condition: .onQueue(queue))
            expect1.fulfill()
        }

        queue.async {
            expect2.fulfill()
        }

        wait(for: [expect1, expect2], timeout: 1, enforceOrder: true)
    }

    func testOptionalMapNoneWithDefaultValue() {
        let value: Int? = nil

        value.asPromise()
            .map(defaultValue: 1) { _ in
                return 2
            }.observe { completion in
                XCTAssertEqual(completion.unwrappedValue, 1)
            }
    }

    func testOptionalMapSomeWithDefaultValue() {
        let value: Int? = 0

        value.asPromise()
            .map(defaultValue: 1) { _ in
                return 2
            }.observe { completion in
                XCTAssertEqual(completion.unwrappedValue, 2)
            }
    }

    func testRunOnOperationQueue() {
        let operationQueue = OperationQueue()
        operationQueue.name = "SerialOperationQueue"
        operationQueue.maxConcurrentOperationCount = 1

        let expect1 = expectation(description: "Wait for the first promise")
        let expect2 = expectation(description: "Wait for the second promise")

        Promise(value: 1)
            .receive(on: .main, after: .milliseconds(100), timerType: .deadline)
            .run(on: operationQueue)
            .observe { completion in
                expect1.fulfill()
            }

        Promise(value: 2)
            .run(on: operationQueue)
            .observe { completion in
                expect2.fulfill()
            }

        wait(for: [expect1, expect2], timeout: 1, enforceOrder: true)
    }

    func testRunOnOperationQueueWithExcusiveCategory() {
        let operationQueue = OperationQueue()
        operationQueue.name = "ConcurrentOperationQueue"

        let expect1 = expectation(description: "Wait for the first promise")
        let expect2 = expectation(description: "Wait for the second promise")

        Promise(value: 1)
            .receive(on: .main, after: .milliseconds(100), timerType: .deadline)
            .run(on: operationQueue, categories: ["MutuallyExclusive"])
            .observe { completion in
                expect1.fulfill()
            }

        Promise(value: 2)
            .run(on: operationQueue, categories: ["MutuallyExclusive"])
            .observe { completion in
                expect2.fulfill()
            }

        wait(for: [expect1, expect2], timeout: 1, enforceOrder: true)
    }

    func testExecutingPromiseCancellation() throws {
        let cancelExpectation = expectation(description: "Expect cancellation handler to trigger")
        let completionExpectation = expectation(description: "Expect promise to complete")

        let promise = Promise<Int> { resolver in
            let work = DispatchWorkItem {
                XCTFail()
                resolver.resolve(value: 1)
            }

            resolver.setCancelHandler {
                work.cancel()
                cancelExpectation.fulfill()

                // Resolve promise since `work` is cancelled now.
                resolver.resolve(completion: .cancelled)
            }

            DispatchQueue.main.async(execute: work)
        }

        promise.observe { completion in
            XCTAssertEqual(completion, .cancelled)
            completionExpectation.fulfill()
        }

        promise.cancel()

        wait(for: [cancelExpectation, completionExpectation], timeout: 1, enforceOrder: true)
    }

    func testPendingPromiseCancellation() {
        let completionExpectation = expectation(description: "Expect promise to complete")

        let promise = Promise.deferred { () -> Int in
            XCTFail()
            return 1
        }

        promise.cancel()

        promise.observe { completion in
            XCTAssertEqual(completion, .cancelled)
            completionExpectation.fulfill()
        }

        wait(for: [completionExpectation], timeout: 1)
    }

    func testUnhandledCancellation() {
        let expectObserve = expectation(description: "Wait for observer")
        let expectCancelHandler = expectation(description: "Wait for cancellation handler")
        let expectResolve = expectation(description: "Wait for resolver")

        let promise = Promise<Bool> { resolver in
            resolver.setCancelHandler {
                expectCancelHandler.fulfill()
                // Do nothing and let the promise continue execution.
            }

            DispatchQueue.main.async {
                expectResolve.fulfill()

                // Resolve the cancelling promise. This should yield the `.cancelled` completion anyway.
                resolver.resolve(value: true)
            }
        }

        promise.observe { completion in
            XCTAssertEqual(completion, .cancelled)
            expectObserve.fulfill()
        }

        promise.cancel()

        wait(for: [expectCancelHandler, expectResolve, expectObserve], timeout: 1, enforceOrder: true)
    }

    func testShouldNotPropagateCancellation() {
        let expectParentCancel = expectation(description: "Parent cancellation handler should never trigger")
        expectParentCancel.isInverted = true

        let expectChildCompletion = expectation(description: "Wait for child to complete")

        let parent = Promise<Int> { resolver in
            resolver.setCancelHandler {
                expectParentCancel.fulfill()
            }

            DispatchQueue.main.async {
                resolver.resolve(value: 1)
            }
        }

        let child = Promise<Int>(parent: parent) { resolver in
            parent.observe { completion in
                resolver.resolve(completion: completion)
            }
        }

        _ = child.setShouldPropagateCancellation(false)

        child.observe { completion in
            XCTAssertEqual(completion, .cancelled)
            expectChildCompletion.fulfill()
        }

        child.cancel()

        wait(for: [expectParentCancel, expectChildCompletion], timeout: 1)
    }

}
