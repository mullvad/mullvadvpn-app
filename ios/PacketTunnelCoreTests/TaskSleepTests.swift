//
//  TaskSleepTests.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 25/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

@testable import PacketTunnelCore
import XCTest

final class TaskSleepTests: XCTestCase {
    func testCancellation() async throws {
        let task = Task {
            try await Task.sleepUsingContinuousClock(for: .seconds(1))
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
        for _ in 0 ... 20 {
            var task = recoveryTask()
            try await Task.sleep(duration: .milliseconds(10))
            task.doAnythingToSilenceAWarning()
        }
    }

    private func recoveryTask() -> AutoCancellingTask {
        AutoCancellingTask(Task.detached {
            while Task.isCancelled == false {
                try await Task.sleepUsingContinuousClock(for: .milliseconds(10))
            }
        })
    }
}

private extension AutoCancellingTask {
    func doAnythingToSilenceAWarning() {}
}
