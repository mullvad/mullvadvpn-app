//
//  BackgroundAsyncTaskObserver.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-07-09.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
import UIKit

@available(iOSApplicationExtension, unavailable)
public final class BackgroundAsyncTaskObserver: AsyncTaskObserver, @unchecked Sendable {
    public let name: String
    public let backgroundTaskProvider: BackgroundTaskProviding
    public let cancelUponExpiration: Bool

    private var taskIdentifier: UIBackgroundTaskIdentifier?

    public init(
        backgroundTaskProvider: BackgroundTaskProviding,
        name: String,
        cancelUponExpiration: Bool
    ) {
        self.backgroundTaskProvider = backgroundTaskProvider
        self.name = name
        self.cancelUponExpiration = cancelUponExpiration
    }

    public func didAttach(
        cancel: @escaping @Sendable () -> Void
    ) {
        let expirationHandler: (@MainActor @Sendable () -> Void)?

        if cancelUponExpiration {
            expirationHandler = {
                cancel()
            }
        } else {
            expirationHandler = nil
        }

        taskIdentifier = backgroundTaskProvider.beginBackgroundTask(
            withName: name,
            expirationHandler: expirationHandler
        )
    }

    public func didStart() {
        // no-op
    }

    public func didCancel() {
        // no-op
    }

    public func didFinish(error: Error?) {
        if let taskIdentifier {
            backgroundTaskProvider.endBackgroundTask(taskIdentifier)
            self.taskIdentifier = nil
        }
    }
}

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
