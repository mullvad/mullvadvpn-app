//
//  TaskQueue.swift
//  PacketTunnel
//
//  Created by pronebird on 30/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

/**
 Task-based FIFO queue used by `PacketTunnelActor`.

 A task is a unit of asynchronous work. A task can be comprised of multiple async calls. The purpose of `TaskQueue` is to make sure that individual tasks can
 execute their work in a transactional fashion without interlacing or interrupting each others work. You can think of it as a simplified `OperationQueue` for
 coroutines.

 `TaskQueue` also implements a basic relationship management between tasks in form of cancelling adjacent tasks that otherwise undo each others work, such as
 for instance a call to `.stop` is guaranteed to undo work of any preceding task. (See `TaskKind` for more information)
 */
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

        currentTask = SerialTask(kind: kind, task: nextTask)

        if let previousTask, kind.shouldCancel(previousTask.kind) {
            previousTask.task.cancel()
        }

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

        currentTask = SerialTask(kind: kind, task: nextTask)

        if let previousTask, kind.shouldCancel(previousTask.kind) {
            previousTask.task.cancel()
        }

        return await nextTask.value
    }
}

/// Kinds of tasks that `TaskQueue` actor performs.
enum TaskKind: Equatable {
    case start, stop, reconnect, keyRotated, networkReachability
}

private struct SerialTask {
    var kind: TaskKind
    var task: AnyTask
}

private extension TaskKind {
    /**
     Returns `true` if the prior task should be cancelled.

     The following adjacent tasks should result in cancellation of the left-hand side task.

     `.start` → `.stop`
     `.reconnect` → `.stop`
     `.reconnect` → `.reconnect`
     `.networkReachability` → `.networkReachability`
     */
    func shouldCancel(_ prior: TaskKind) -> Bool {
        if self == .stop, prior != .stop {
            // Stop task can cancel any prior task.
            return true
        } else if self == .reconnect, prior == .reconnect {
            // Cancel prior task to reconnect.
            return true
        } else if self == .networkReachability, prior == .networkReachability {
            // Coalesce network reachability changes
            return true
        } else {
            return false
        }
    }
}
