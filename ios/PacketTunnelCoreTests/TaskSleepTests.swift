//
//  TaskSleepTests.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 25/09/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import XCTest

@testable import PacketTunnelCore

final class TaskSleepTests: XCTestCase {
    func testCancellation() async throws {
        let task = Task {
            try await Task.sleep(for: .seconds(1))
        }

        task.cancel()

        do {
            try await task.value

            XCTFail("Task must be cancelled.")
        } catch {
            XCTAssert(error is CancellationError)
        }
    }

    /// This test triggers a race condition in `sleepUsingContinuousClock` where an `AutoCancellingTask` will
    /// cancel a `DispatchSourceTimer` in a `Task` trying to call `resume` on its continuation handler more than once
    func testSuccessfulEventHandlerRemovesCancellation() async throws {
        for _ in 0...20 {
            let task = recoveryTask()
            try await Task.sleep(for: .milliseconds(10))
            task.callDummyFunctionToForceConcurrencyWait()
        }
    }

    private func recoveryTask() -> AutoCancellingTask {
        AutoCancellingTask(
            Task.detached {
                while Task.isCancelled == false {
                    try await Task.sleep(for: .milliseconds(10))
                }
            })
    }
}

private extension AutoCancellingTask {
    /// This function is here to silence a warning about unused variables in `testSuccessfulEventHandlerRemovesCancellation`
    /// The following construct `_ = recoveryTask()` cannot be used as the resulting `AutoCancellingTask`
    /// would immediately get `deinit`ed, changing the test scenario.
    /// A dummy function is needed to make sure the task is not cancelled before concurrency is forced
    /// by having a call to `Task.sleep`
    func callDummyFunctionToForceConcurrencyWait() {}
}
