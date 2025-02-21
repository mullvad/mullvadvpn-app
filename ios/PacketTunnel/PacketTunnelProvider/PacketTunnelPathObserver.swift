//
//  PacketTunnelPathObserver.swift
//  PacketTunnel
//
//  Created by pronebird on 10/08/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadLogging
import MullvadTypes
import Network
import NetworkExtension
import PacketTunnelCore

final class PacketTunnelPathObserver: DefaultPathObserverProtocol, Sendable {
    private let eventQueue: DispatchQueue
    private let pathMonitor: NWPathMonitor
    nonisolated(unsafe) let logger = Logger(label: "PacketTunnelPathObserver")
    private let stateLock = NSLock()

    nonisolated(unsafe) private var started = false

    public var currentPathStatus: Network.NWPath.Status {
        stateLock.withLock {
            pathMonitor.currentPath.status
        }
    }

    init(eventQueue: DispatchQueue) {
        self.eventQueue = eventQueue

        pathMonitor = NWPathMonitor(prohibitedInterfaceTypes: [.other])
    }

    func start(_ body: @escaping @Sendable (Network.NWPath.Status) -> Void) {
        stateLock.withLock {
            guard started == false else { return }
            defer { started = true }
            pathMonitor.pathUpdateHandler = { updatedPath in
                body(updatedPath.status)
            }

            pathMonitor.start(queue: eventQueue)
        }
    }

    func stop() {
        stateLock.withLock {
            guard started == true else { return }
            defer { started = false }
            pathMonitor.pathUpdateHandler = nil
            pathMonitor.cancel()
        }
    }
}
