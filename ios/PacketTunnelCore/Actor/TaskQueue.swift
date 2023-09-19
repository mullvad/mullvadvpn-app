//
//  TaskQueue.swift
//  PacketTunnel
//
//  Created by pronebird on 30/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

/// Internal task identifier type
typealias TaskId = UInt

/**
 Task-based FIFO queue used by `PacketTunnelActor`.

 A task is a unit of asynchronous work. A task can be comprised of multiple async calls. The purpose of `TaskQueue` is to make sure that individual tasks can
 execute their work in a transactional fashion without interlacing or interrupting each others work. You can think of it as a simplified `OperationQueue` for
 coroutines.

 `TaskQueue` also implements a basic relationship management between tasks in form of cancelling adjacent tasks that otherwise undo each others work, such as
 for instance a call to `.stop` is guaranteed to undo work of any preceding task. (See `TaskKind` for more information)
 */
final actor TaskQueue {
    /// Ever incrementing identifier that's is used to distinguish between different tasks.
    private var taskId: TaskId = 0
    private var queuedTasks: [SerialTask] = []

    public init() {}

    public func add<Output>(
        kind: TaskKind,
        priority: TaskPriority? = nil,
        operation: @escaping () async throws -> Output
    ) async throws -> Output {
        let previousTask = queuedTasks.last
        let nextTask = Task(priority: priority) {
            await previousTask?.task.waitForCompletion()

            return try await operation()
        }

        let nextTaskId = registerTask(kind: kind, nextTask: nextTask)
        cancelPrecedingTasksIfNeeded()

        defer {
            unregisterTask(nextTaskId)
        }

        return try await nextTask.value
    }

    private func nextTaskId() -> TaskId {
        let (value, isOverflow) = taskId.addingReportingOverflow(1)
        taskId = isOverflow ? 0 : value
        return UInt(taskId)
    }

    private func registerTask(kind: TaskKind, nextTask: AnyTask) -> TaskId {
        let nextTaskId = nextTaskId()

        queuedTasks.append(SerialTask(id: nextTaskId, kind: kind, task: nextTask))

        return nextTaskId
    }

    private func unregisterTask(_ taskId: TaskId) {
        let index = queuedTasks.firstIndex { $0.id == taskId }

        if let index {
            queuedTasks.remove(at: index)
        }
    }

    private func cancelPrecedingTasksIfNeeded() {
        guard let current = queuedTasks.last else { return }

        let tasksToCancel = queuedTasks.dropLast(1).filter { preceding in
            current.kind.shouldCancel(preceding.kind)
        }

        tasksToCancel.forEach { $0.task.cancel() }
    }
}

/// Kinds of tasks that `TaskQueue` actor performs.
enum TaskKind: Equatable {
    /// Task that starts the tunnel.
    case start

    /// Task that stops the tunnel.
    case stop

    /// Task that reconnects the tunnel.
    case reconnect

    /// Task that enters tunnel into blocked state.
    /// Scheduled via external call in response to device check.
    case blockedState

    //
    case keyRotated
    case networkReachability
}

private struct SerialTask {
    var id: TaskId
    var kind: TaskKind
    var task: AnyTask
}

private extension TaskKind {
    /**
     Returns `true` if the prior task should be cancelled.

     - `.stop` task cancels all tasks in the queue except other `.stop` tasks.
     - `.reconnect` task can cancel any prior `.reconnect`.
     */
    func shouldCancel(_ other: TaskKind) -> Bool {
        if self == .stop, other != .stop {
            // Stop task can cancel any prior task.
            return true
        } else if self == .reconnect, other == .reconnect {
            // Cancel prior task to reconnect.
            return true
        } else {
            return false
        }
    }
}
