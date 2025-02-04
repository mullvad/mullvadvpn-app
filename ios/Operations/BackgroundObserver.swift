//
//  BackgroundObserver.swift
//  Operations
//
//  Created by pronebird on 31/05/2022.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

#if canImport(UIKit)

import MullvadTypes
import UIKit

@available(iOSApplicationExtension, unavailable)
public final class BackgroundObserver: OperationObserver {
    public let name: String
    public let backgroundTaskProvider: BackgroundTaskProviding
    public let cancelUponExpiration: Bool

    private var taskIdentifier: UIBackgroundTaskIdentifier?

    public init(backgroundTaskProvider: BackgroundTaskProviding, name: String, cancelUponExpiration: Bool) {
        self.backgroundTaskProvider = backgroundTaskProvider
        self.name = name
        self.cancelUponExpiration = cancelUponExpiration
    }

    public func didAttach(to operation: Operation) {
        let expirationHandler = cancelUponExpiration
            ? { @MainActor in operation.cancel() } as? @MainActor @Sendable () -> Void
            : nil

        taskIdentifier = backgroundTaskProvider.beginBackgroundTask(
            withName: name,
            expirationHandler: expirationHandler
        )
    }

    public func operationDidStart(_ operation: Operation) {
        // no-op
    }

    public func operationDidCancel(_ operation: Operation) {
        // no-op
    }

    public func operationDidFinish(_ operation: Operation, error: Error?) {
        if let taskIdentifier {
            backgroundTaskProvider.endBackgroundTask(taskIdentifier)
        }
    }
}

#endif
