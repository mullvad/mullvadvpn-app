// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
