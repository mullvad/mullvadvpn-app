//
//  AutoCancellingTask.swift
//  PacketTunnel
//
//  Created by pronebird on 31/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Type that cancels the given task upon `deinit` (identical to `Combine.AnyCancellable`), unless explicitly told not to.
public final class AutoCancellingTask {
    private var task: AnyTask?
    private let taskLock = NSLock()

    init(_ task: AnyTask) {
        self.task = task
    }

    /**
     Forget the task held internally to prevent cancellation on deinit.

     This is particularly useful when the task is already executing and does not want to be interrupted.
     */
    func disableCancellation() {
        taskLock.withLock {
            task = nil
        }
    }

    deinit {
        taskLock.withLock {
            task?.cancel()
        }
    }
}
