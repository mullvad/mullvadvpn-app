//
//  TaskQueue.swift
//  PacketTunnel
//
//  Created by pronebird on 30/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

// Kinds of tasks that actor performs.
enum TaskKind: Equatable {
    case start, stop, reconnect
}

/// Task-based FIFO queue.
final actor TaskQueue {
    private var currentTask: SerialTask?

    public init() {}

    public func add<Output>(
        kind: TaskKind,
        priority: TaskPriority? = nil,
        operation: @escaping () async throws -> Output
    ) async throws -> Output {
        let previousTask = currentTask
        let nextTask = Task(priority: priority) {
            await previousTask?.task.waitForCompletion()

            return try await operation()
        }

        setCurrent(SerialTask(kind: kind, task: nextTask))

        return try await nextTask.value
    }

    public func add<Output>(
        kind: TaskKind,
        priority: TaskPriority? = nil,
        operation: @escaping () async -> Output
    ) async -> Output {
        let previousTask = currentTask
        let nextTask = Task(priority: priority) {
            await previousTask?.task.waitForCompletion()

            return await operation()
        }

        setCurrent(SerialTask(kind: kind, task: nextTask))

        return await nextTask.value
    }

    private func setCurrent(_ nextTask: SerialTask) {
        let previousTask = currentTask

        currentTask = nextTask

        if let previousTask, nextTask.kind.shouldCancel(previousTask.kind) {
            previousTask.task.cancel()
        }
    }
}

private struct SerialTask {
    var kind: TaskKind
    var task: AnyTask
}

private extension TaskKind {
    /// Returns `true` if the prior task should be cancelled.
    func shouldCancel(_ prior: TaskKind) -> Bool {
        if self == .stop, prior != .stop {
            // Stop task can cancel any prior task.
            return true
        } else if self == .reconnect, prior == .reconnect {
            // Cancel prior task to reconnect.
            return true
        } else {
            return false
        }
    }
}
