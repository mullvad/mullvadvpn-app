//
//  SerialQueueTests.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-08-10.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Testing

struct SerialQueueTests {
    let testQueue = DispatchQueue(label: "SerialQueueTests.testQueue")

    @Test
    func testOperationsExecuteSerially() async throws {
        let queue = SerialQueue()
        let counter = Counter()
        async let first: Void = queue.enqueue(completionQueue: testQueue) {
            await counter.begin()
            try await Task.sleep(for: .milliseconds(3))
            await counter.end()
        }

        async let second: Void = queue.enqueue(completionQueue: testQueue) {
            await counter.begin()
            try await Task.sleep(for: .milliseconds(2))
            await counter.end()
        }

        async let third: Void = queue.enqueue(completionQueue: testQueue) {
            await counter.begin()
            try await Task.sleep(for: .milliseconds(1))
            await counter.end()
        }

        _ = try await (first, second, third)

        #expect(await counter.maxRunning() == 1)
    }

    @Test
    func resultsAreObservedInEnqueueOrder() async throws {
        let queue = SerialQueue()
        let length = 10
        let results = try await withThrowingTaskGroup(of: (Int, Int).self) { group in
            for value in 1...length {
                group.addTask {
                    let result = try await queue.enqueue(completionQueue: testQueue) {
                        if value == 1 {
                            try await Task.sleep(for: .milliseconds(2))
                        }
                        return value
                    }
                    return (value, result)
                }
            }

            var collected = Array(repeating: 0, count: length)
            for try await (index, result) in group { collected[index - 1] = result }
            return collected
        }
        #expect(results == Array(1...length))
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
