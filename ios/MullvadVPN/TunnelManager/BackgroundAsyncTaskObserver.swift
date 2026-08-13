//
//  BackgroundAsyncTaskObserver.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-07-09.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

@available(iOSApplicationExtension, unavailable)
public func withBackgroundTask<T: Sendable>(
    backgroundTaskProvider: BackgroundTaskProviding,
    name: String,
    cancelUponExpiration: Bool,
    operation: @escaping @Sendable () async throws -> T
) async throws -> T {
    let cancellationBox = CancellationBox()

    let expirationHandler: (@MainActor @Sendable () -> Void)?

    if cancelUponExpiration {
        expirationHandler = {
            cancellationBox.cancel()
        }
    } else {
        expirationHandler = nil
    }

    let taskIdentifier = backgroundTaskProvider.beginBackgroundTask(
        withName: name,
        expirationHandler: expirationHandler
    )

    defer {
        backgroundTaskProvider.endBackgroundTask(taskIdentifier)
    }

    try cancellationBox.checkCancellation()
    return try await operation()
}

private final class CancellationBox: @unchecked Sendable {
    private var isCancelled: Bool = false
    private var lock = NSLock()

    func cancel() {
        lock.lock()
        isCancelled = true
        lock.unlock()
    }

    func checkCancellation() throws {
        lock.lock()
        let isCancelled = self.isCancelled
        lock.unlock()
        guard isCancelled else { return }
        throw CancellationError()
    }
}
