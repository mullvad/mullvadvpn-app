//
//  AsyncSerialExecutorTests.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-08-10.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Testing

struct AsyncSerialExecutorTests {

    @Test
    func testOperationsExecuteSerially() async throws {
        let executor = AsyncSerialExecutor()
        let counter = Counter()

        async let first: Void = executor.enqueue {
            await counter.begin()
            try await Task.sleep(for: .milliseconds(10))
            await counter.end()
        }

        async let second: Void = executor.enqueue {
            await counter.begin()
            try await Task.sleep(for: .milliseconds(2))
            await counter.end()
        }

        async let third: Void = executor.enqueue {
            await counter.begin()
            try await Task.sleep(for: .milliseconds(5))
            await counter.end()
        }

        _ = try await (first, second, third)

        #expect(await counter.maxRunning() == 1)
    }
}

private actor Counter {
    private var running = 0
    private var maximum = 0

    func begin() {
        running += 1
        maximum = max(maximum, running)
    }

    func end() {
        running -= 1
    }

    func maxRunning() -> Int {
        maximum
    }
}
