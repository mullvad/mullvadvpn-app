//
//  AnyTask.swift
//  PacketTunnel
//
//  Created by pronebird on 28/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Protocol describing a type-erased `Task`.
public protocol AnyTask {
    /// Wait for task to complete execution.
    func waitForCompletion() async

    /// Cancel task.
    func cancel()
}

extension Task: AnyTask {
    public func waitForCompletion() async {
        _ = try? await value
    }
}
