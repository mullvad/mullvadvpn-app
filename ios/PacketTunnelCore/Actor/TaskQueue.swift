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

 `TaskQueue` also implements a basic relationship management between tasks in form of cancelling running tasks that otherwise undo each others work, such as
 for instance a call to `.stop` is guaranteed to undo work of any preceding task. (See `TaskKind` for more information)
 */
actor TaskQueue {
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

    /// Returns next task ID, then increments internal task counter.
    private func nextTaskId() -> TaskId {
        let currentTaskId = taskId
        let (value, isOverflow) = taskId.addingReportingOverflow(1)

        taskId = isOverflow ? 0 : value

        return currentTaskId
    }

    /// Register new task in the array of all tasks that are executing or pending execution.
    /// Returns new task ID.
    private func registerTask(kind: TaskKind, nextTask: AnyTask) -> TaskId {
        let serialTask = SerialTask(id: nextTaskId(), kind: kind, task: nextTask)
        queuedTasks.append(serialTask)
        return serialTask.id
    }

    /// Remove task by ID previously returned by call to `registerTask()`.
    private func unregisterTask(_ taskId: TaskId) {
        let index = queuedTasks.firstIndex { $0.id == taskId }

        if let index {
            queuedTasks.remove(at: index)
        }
    }

    /// Check if new task in the queue may trigger cancellation of preceding tasks.
    private func cancelPrecedingTasksIfNeeded() {
        guard let current = queuedTasks.last else { return }

        // Reverse the list of tasks so that we cancel tasks that haven't started yet first.
        let tasksToCancel = queuedTasks.dropLast(1)
            .reversed()
            .filter { preceding in
                return current.shouldCancel(preceding)
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

    /// <TBD>
    case keyRotated

    /// <TBD>
    case networkReachability
}

private struct SerialTask {
    /// Internal task identifier used to identify individual tasks.
    var id: TaskId

    /// Kind of task.
    var kind: TaskKind

    /// A type erased copy of a concrete `Task` that can be used to wait for task completion or cancel it.
    var task: AnyTask

    /**
     Returns `true` if the other task should be cancelled.

     - `.stop` task cancels all tasks in the queue except other `.stop` tasks.
     - `.reconnect` task can cancel any prior `.reconnect`.
     */
    func shouldCancel(_ other: SerialTask) -> Bool {
        if kind == .stop, other.kind != .stop {
            // Stop task can cancel any prior task.
            return true
        }

        if kind == .reconnect, other.kind == .reconnect {
            // Cancel prior task to reconnect.
            return true
        }

        return false
    }
}
