//
//  AutoCancellingTask.swift
//  PacketTunnel
//
//  Created by pronebird on 31/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/**
 Type that cancels the given task upon `deinit`.

 It behaves identical to `Combine.AnyCancellable`.
 */
public final class AutoCancellingTask {
    private let task: AnyTask

    public init(_ task: AnyTask) {
        self.task = task
    }

    deinit {
        task.cancel()
    }
}
