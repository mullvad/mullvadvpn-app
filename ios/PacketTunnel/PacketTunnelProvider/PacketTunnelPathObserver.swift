//
//  PacketTunnelPathObserver.swift
//  PacketTunnel
//
//  Created by pronebird on 10/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
import NetworkExtension
import PacketTunnelCore

final class PacketTunnelPathObserver: DefaultPathObserverProtocol {
    private weak var packetTunnelProvider: NEPacketTunnelProvider?
    private let stateLock = NSLock()
    private var pathUpdatePublisher: AnyCancellable?
    private let eventQueue: DispatchQueue

    init(packetTunnelProvider: NEPacketTunnelProvider, eventQueue: DispatchQueue) {
        self.packetTunnelProvider = packetTunnelProvider
        self.eventQueue = eventQueue
    }

    var defaultPath: NetworkPath? {
        return packetTunnelProvider?.defaultPath
    }

    func start(_ body: @escaping (NetworkPath) -> Void) {
        stateLock.withLock {
            pathUpdatePublisher?.cancel()

            // Normally packet tunnel provider should exist throughout the network extension lifetime.
            pathUpdatePublisher = packetTunnelProvider?.publisher(for: \.defaultPath)
                .removeDuplicates(by: { oldPath, newPath in
                    oldPath?.status == newPath?.status
                })
                .throttle(for: .seconds(2), scheduler: eventQueue, latest: true)
                .sink { change in
                    if let change {
                        body(change)
                    }
                }
        }
    }

    func stop() {
        stateLock.withLock {
            pathUpdatePublisher?.cancel()
            pathUpdatePublisher = nil
        }
    }
}

extension NetworkExtension.NWPath: NetworkPath {}
