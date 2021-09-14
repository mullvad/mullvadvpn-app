//
//  Promise+BackgroundTask.swift
//  Promise+BackgroundTask
//
//  Created by pronebird on 06/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension Promise {

    /// Start the background task for the duration of the upstream execution.
    func requestBackgroundTime(taskName: String? = nil) -> Promise<Value> {
        return Promise<Value> { resolver in
            var backgroundTaskIdentifier: UIBackgroundTaskIdentifier?

            let beginBackgroundTask = {
                backgroundTaskIdentifier = UIApplication.shared.beginBackgroundTask(withName: taskName) {
                    resolver.resolve(completion: .cancelled)
                }
            }

            let endBackgroundTask = {
                guard let taskIdentifier = backgroundTaskIdentifier,
                      taskIdentifier != .invalid else { return }

                UIApplication.shared.endBackgroundTask(taskIdentifier)
                backgroundTaskIdentifier = nil
            }

            let endBackgroundTaskOnMainQueue = {
                if Thread.isMainThread {
                    endBackgroundTask()
                } else {
                    DispatchQueue.main.async(execute: endBackgroundTask)
                }
            }

            if Thread.isMainThread {
                beginBackgroundTask()
            } else {
                DispatchQueue.main.async(execute: beginBackgroundTask)
            }

            resolver.setCancelHandler {
                endBackgroundTaskOnMainQueue()
            }

            self.observe { completion in
                resolver.resolve(completion: completion)

                endBackgroundTaskOnMainQueue()
            }
        }
    }
}
