// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadLogging
import Network

public final class PacketTunnelPathObserver: DefaultPathObserverProtocol, Sendable {
    private let eventQueue: DispatchQueue
    private let pathMonitor = NWPathMonitor()
    let logger = Logger(label: "PacketTunnelPathObserver")
    private let stateLock = NSLock()

    nonisolated(unsafe) private var started = false
    nonisolated(unsafe) private var pendingPathUpdate: DispatchWorkItem?
    private static let pathUpdateDebounceDelay: DispatchTimeInterval = .seconds(2)

    public var currentPathStatus: Network.NWPath.Status {
        stateLock.withLock {
            pathMonitor.currentPath.status
        }
    }

    public init(eventQueue: DispatchQueue) {
        self.eventQueue = eventQueue
    }

    public func start(_ body: @escaping @Sendable (Network.NWPath.Status) -> Void) {
        stateLock.withLock {
            guard started == false else { return }
            defer { started = true }
            pathMonitor.pathUpdateHandler = { [weak self] updatedPath in
                guard let self else { return }
                self.stateLock.withLock {
                    self.pendingPathUpdate?.cancel()

                    let workItem = DispatchWorkItem {
                        body(updatedPath.status)
                    }
                    self.pendingPathUpdate = workItem

                    self.eventQueue.asyncAfter(
                        deadline: .now() + Self.pathUpdateDebounceDelay,
                        execute: workItem
                    )
                }
            }

            pathMonitor.start(queue: eventQueue)
        }
    }

    public func stop() {
        stateLock.withLock {
            guard started == true else { return }
            defer { started = false }
            pendingPathUpdate?.cancel()
            pendingPathUpdate = nil
            pathMonitor.pathUpdateHandler = nil
            pathMonitor.cancel()
        }
    }
}
