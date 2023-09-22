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
}
