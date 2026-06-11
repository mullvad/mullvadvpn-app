//
//  PathMonitor.swift
//  MullvadVPN
//
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Network
import OSLog

final class PathMonitor {
    private let monitor: NWPathMonitor
    private let queue = DispatchQueue(label: "com.test.pathmonitor", qos: .utility)
    private let _snapshots = OSAllocatedUnfairLock<[NetworkPathSnapshot]>(initialState: [])

    var snapshots: [NetworkPathSnapshot] { _snapshots.withLock { $0 } }
    let onPathChange: (@Sendable (NetworkPathSnapshot) -> Void)?

    init(onPathChange: (@Sendable (NetworkPathSnapshot) -> Void)?) {
        self.onPathChange = onPathChange
        self.monitor = NWPathMonitor()

        let snapshots = _snapshots
        let handler = onPathChange

        self.monitor.pathUpdateHandler = { path in
            let snap = NetworkPathSnapshot(path)
            snapshots.withLock { $0.append(snap) }
            handler?(snap)
        }
    }

    func start() {
        monitor.pathUpdateHandler?(monitor.currentPath)
        monitor.start(queue: queue)
    }

    func cancel() { monitor.cancel() }

    deinit { monitor.cancel() }
}
