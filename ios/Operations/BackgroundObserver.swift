//
//  BackgroundObserver.swift
//  Operations
//
//  Created by pronebird on 31/05/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

#if canImport(UIKit)

import UIKit

@available(iOSApplicationExtension, unavailable)
public final class BackgroundObserver: OperationObserver {
    public let name: String
    public let application: UIApplication
    public let cancelUponExpiration: Bool

    private var taskIdentifier: UIBackgroundTaskIdentifier?

    public init(application: UIApplication, name: String, cancelUponExpiration: Bool) {
        self.application = application
        self.name = name
        self.cancelUponExpiration = cancelUponExpiration
    }

    public func didAttach(to operation: Operation) {
        let expirationHandler = cancelUponExpiration ? { operation.cancel() } : nil

        taskIdentifier = application.beginBackgroundTask(
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
        if let taskIdentifier = taskIdentifier {
            application.endBackgroundTask(taskIdentifier)
        }
    }
}

#endif
