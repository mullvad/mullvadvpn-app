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
    let task = Task {
        try await operation()
    }

    let expirationHandler: (@MainActor @Sendable () -> Void)?

    if cancelUponExpiration {
        expirationHandler = {
            task.cancel()
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

    return try await task.value
}
