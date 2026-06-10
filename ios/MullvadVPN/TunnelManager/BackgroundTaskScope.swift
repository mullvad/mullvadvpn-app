//
//  BackgroundTaskScope.swift
//  MullvadVPN
//
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import UIKit

/// Runs `body` inside a UIKit background task, the async counterpart of `BackgroundObserver`
/// with `cancelUponExpiration: true`: when the system revokes background execution time, the
/// task running `body` is cancelled. Cancellation of the calling task is forwarded to `body`,
/// and the background task is always ended on exit.
@available(iOSApplicationExtension, unavailable)
func withBackgroundTask<Success: Sendable>(
    name: String,
    provider: BackgroundTaskProviding,
    body: @escaping @Sendable () async throws -> Success
) async throws -> Success {
    // The expiration handler must be registered before `body` starts, but it needs a handle
    // to the not-yet-created task — hence the box, which also covers expiration firing
    // before the task is set.
    let cancelBox = CancelBox()

    let identifier = provider.beginBackgroundTask(withName: name) {
        cancelBox.cancel()
    }
    defer { provider.endBackgroundTask(identifier) }

    // `body` runs in an unstructured task so the expiration handler can cancel it. This
    // loses priority inheritance from the caller, which is acceptable here.
    let task = Task { try await body() }
    cancelBox.set { task.cancel() }

    return try await withTaskCancellationHandler {
        try await task.value
    } onCancel: {
        task.cancel()
    }
}

/// Holds a cancel handler that may be invoked before it is set.
private final class CancelBox: @unchecked Sendable {
    private let lock = NSLock()
    private var isCancelled = false
    private var handler: (@Sendable () -> Void)?

    func set(_ newHandler: @escaping @Sendable () -> Void) {
        lock.lock()
        if isCancelled {
            lock.unlock()
            newHandler()
        } else {
            handler = newHandler
            lock.unlock()
        }
    }

    func cancel() {
        lock.lock()
        isCancelled = true
        let pendingHandler = handler
        handler = nil
        lock.unlock()

        pendingHandler?()
    }
}
