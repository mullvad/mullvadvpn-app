//
//  TaskQueueTests.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 14/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

@testable import PacketTunnelCore
import XCTest

final class TaskQueueTests: XCTestCase {
    func testSerialTaskExecution() async throws {
        let queue = TaskQueue()

        let firstExpectation = expectation(description: "Complete first task")
        let secondExpectation = expectation(description: "Complete second task")

        async let _ = queue.add(kind: TaskKind.start) {
            try await Task.sleep(seconds: 1)
            firstExpectation.fulfill()
        }
        async let _ = queue.add(kind: TaskKind.start) {
            secondExpectation.fulfill()
        }

        await fulfillment(of: [firstExpectation, secondExpectation], timeout: 2, enforceOrder: true)
    }

    func testReconnectShouldNotCancelPrecedingStart() async throws {
        let queue = TaskQueue()

        async let start: () = queue.add(kind: TaskKind.start) {
            try await Task.sleep(seconds: 1)
        }
        async let _ = queue.add(kind: TaskKind.reconnect) {}

        do {
            try await start
        } catch {
            XCTFail("Start task cannot be cancelled by reconnect task")
        }
    }

    func testReconnectShouldCancelPrecedingReconnect() async throws {
        let queue = TaskQueue()

        async let first: () = queue.add(kind: TaskKind.reconnect) {
            try await Task.sleep(seconds: 1)
        }
        async let _ = queue.add(kind: TaskKind.reconnect) {}

        do {
            try await first
            XCTFail("Reconnect task must be cancelled!")
        } catch {}
    }

    func testStopShouldCancelPrecedingStart() async throws {
        let queue = TaskQueue()

        async let first: () = queue.add(kind: TaskKind.start) {
            try await Task.sleep(seconds: 1)
        }
        async let _ = queue.add(kind: TaskKind.stop) {}

        do {
            try await first
            XCTFail("Start task must be cancelled!")
        } catch {}
    }
}
