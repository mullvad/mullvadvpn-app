//
//  TransportMonitor.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-10-07.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST

final class TransportMonitor {
    private let tunnelManager: TunnelManager
    private let tunnelStore: TunnelStore
    private let urlSessionTransport: REST.URLSessionTransport

    init(tunnelManager: TunnelManager, tunnelStore: TunnelStore) {
        self.tunnelManager = tunnelManager
        self.tunnelStore = tunnelStore

        urlSessionTransport = REST.URLSessionTransport(urlSession: REST.makeURLSession())
    }

    var transport: RESTTransport? {
        let tunnel = tunnelStore.getPersistentTunnels().first { tunnel in
            return tunnel.status == .connecting ||
                tunnel.status == .reasserting ||
                tunnel.status == .connected
        }

        if let tunnel = tunnel, shouldByPassVPN(tunnel: tunnel) {
            return PacketTunnelTransport(tunnel: tunnel)
        } else {
            return urlSessionTransport
        }
    }

    private func shouldByPassVPN(tunnel: Tunnel) -> Bool {
        switch tunnel.status {
        case .connected:
            return tunnelManager.isConfigurationLoaded && tunnelManager.deviceState == .revoked

        case .connecting, .reasserting:
            return true

        default:
            return false
        }
    }
}
