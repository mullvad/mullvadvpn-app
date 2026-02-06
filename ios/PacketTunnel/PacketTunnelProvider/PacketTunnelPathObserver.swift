//
//  PacketTunnelPathObserver.swift
//  PacketTunnel
//
//  Created by pronebird on 10/08/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadLogging
import MullvadTypes
import Network
import NetworkExtension
import PacketTunnelCore

final class PacketTunnelPathObserver: DefaultPathObserverProtocol, Sendable {
    private let eventQueue: DispatchQueue
    private let pathMonitor = NWPathMonitor()
    nonisolated(unsafe) let logger = Logger(label: "PacketTunnelPathObserver")
    private let stateLock = NSLock()

    nonisolated(unsafe) private var started = false
    nonisolated(unsafe) private var pendingPathUpdate: DispatchWorkItem?
    private static let pathUpdateDebounceDelay: DispatchTimeInterval = .seconds(2)

    public var currentPathStatus: Network.NWPath.Status {
        stateLock.withLock {
            pathMonitor.currentPath.status
        }
    }

    init(eventQueue: DispatchQueue) {
        self.eventQueue = eventQueue
    }

    func start(_ body: @escaping @Sendable (Network.NWPath.Status) -> Void) {
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

    func stop() {
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
