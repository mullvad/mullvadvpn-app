//
//  PacketTunnelPathObserver.swift
//  PacketTunnel
//
//  Created by pronebird on 10/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension
import PacketTunnelCore

final class PacketTunnelPathObserver: DefaultPathObserverProtocol {
    private weak var packetTunnelProvider: NEPacketTunnelProvider?
    private let stateLock = NSLock()
    private var observationToken: NSKeyValueObservation?

    init(packetTunnelProvider: NEPacketTunnelProvider) {
        self.packetTunnelProvider = packetTunnelProvider
    }

    var defaultPath: NetworkPath? {
        return packetTunnelProvider?.defaultPath
    }

    func start(_ body: @escaping (NetworkPath) -> Void) {
        stateLock.withLock {
            observationToken?.invalidate()

            // Normally packet tunnel provider should exist throughout the network extension lifetime.
            observationToken = packetTunnelProvider?.observe(\.defaultPath, options: [.new]) { _, change in
                let nwPath = change.newValue.flatMap { $0 }
                if let nwPath {
                    body(nwPath)
                }
            }
        }
    }

    func stop() {
        stateLock.withLock {
            observationToken?.invalidate()
            observationToken = nil
        }
    }
}

extension NetworkExtension.NWPath: NetworkPath {}
