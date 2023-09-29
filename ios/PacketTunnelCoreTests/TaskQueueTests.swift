//
//  TaskQueueTests.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 14/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

@testable import PacketTunnelCore
import WireGuardKitTypes
import XCTest

final class TaskQueueTests: XCTestCase {
    func testSerialTaskExecution() async throws {
        let queue = TaskQueue()

        let firstExpectation = expectation(description: "Complete first task")
        let secondExpectation = expectation(description: "Complete second task")

        async let _ = queue.add(kind: .start) {
            try await Task.sleep(duration: .seconds(1))
            firstExpectation.fulfill()
        }
        async let _ = queue.add(kind: .start) {
            secondExpectation.fulfill()
        }

        await fulfillment(of: [firstExpectation, secondExpectation], timeout: 2, enforceOrder: true)
    }

    func testReconnectShouldNotCancelPrecedingStart() async throws {
        let queue = TaskQueue()

        async let start: () = queue.add(kind: .start) {
            try await Task.sleep(duration: .seconds(1))
        }
        async let _ = queue.add(kind: .reconnect) {}

        do {
            try await start
        } catch {
            XCTFail("Start task cannot be cancelled by reconnect task")
        }
    }

    func testCoalesceCallsToReconnect() async throws {
        let queue = TaskQueue()

        async let first: () = queue.add(kind: .reconnect) {
            try await Task.sleep(duration: .seconds(1))
        }
        async let _ = queue.add(kind: .networkReachability) {
            try await Task.sleep(duration: .seconds(1))
        }
        async let _ = queue.add(kind: .reconnect) {}

        do {
            try await first
            XCTFail("Preceding reconnection task must be cancelled!")
        } catch {}
    }

    func testStopShouldCancelPrecedingTasks() async throws {
        let queue = TaskQueue()

        async let first: () = queue.add(kind: .start) {
            try await Task.sleep(duration: .seconds(1))
        }
        async let second: () = queue.add(kind: .reconnect) {
            try await Task.sleep(duration: .seconds(1))
        }
        async let _ = queue.add(kind: .stop) {}

        do {
            _ = try await (first, second)
            XCTFail("Preceding tasks must be cancelled!")
        } catch {}
    }
}
