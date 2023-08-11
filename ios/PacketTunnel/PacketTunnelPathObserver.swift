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

    init(packetTunnelProvider: NEPacketTunnelProvider) {
        self.packetTunnelProvider = packetTunnelProvider
    }

    var defaultPath: NetworkPath? {
        return packetTunnelProvider?.defaultPath
    }

    func observe(_ body: @escaping (NetworkPath) -> Void) -> DefaultPathObservation {
        let observation = packetTunnelProvider?.observe(\.defaultPath, options: [.new]) { _, change in
            let nwPath = change.newValue.flatMap { $0 }
            if let nwPath {
                body(nwPath)
            }
        }
        return observation ?? EmptyDefaultPathObservation()
    }
}

extension NetworkExtension.NWPath: NetworkPath {}
extension NSKeyValueObservation: DefaultPathObservation {}

private struct EmptyDefaultPathObservation: DefaultPathObservation {
    func invalidate() {}
}
