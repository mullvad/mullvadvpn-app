//
//  BackgroundObserver.swift
//  MullvadVPN
//
//  Created by pronebird on 31/05/2022.
//

#if canImport(UIKit)

import UIKit

class BackgroundObserver: OperationObserver {
    let name: String
    let application: UIApplication
    let cancelUponExpiration: Bool

    private var taskIdentifier: UIBackgroundTaskIdentifier?

    init(
        application: UIApplication = .shared,
        name: String,
        cancelUponExpiration: Bool
    )
    {
        self.application = application
        self.name = name
        self.cancelUponExpiration = cancelUponExpiration
    }

    func didAttach(to operation: Operation) {
        let expirationHandler = cancelUponExpiration ? { operation.cancel() } : nil

        taskIdentifier = application.beginBackgroundTask(
            withName: name,
            expirationHandler: expirationHandler
        )
    }

    func operationDidStart(_ operation: Operation) {
        // no-op
    }

    func operationDidCancel(_ operation: Operation) {
        // no-op
    }

    func operationDidFinish(_ operation: Operation, error: Error?) {
        if let taskIdentifier = taskIdentifier {
            application.endBackgroundTask(taskIdentifier)
        }
    }
}

#endif
