//
//  TransportMonitor.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-10-07.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadTransport
import RelayCache
import RelaySelector

final class TransportMonitor: RESTTransportProvider {
    private let tunnelManager: TunnelManager
    private let tunnelStore: TunnelStore
    private let transportProvider: TransportProvider

    // MARK: -

    // MARK: Public API


    init(tunnelManager: TunnelManager, tunnelStore: TunnelStore, transportProvider: TransportProvider) {
        self.tunnelManager = tunnelManager
        self.tunnelStore = tunnelStore
        self.transportProvider = transportProvider
    }

    public func transport() -> RESTTransport? {
        return selectTransport(transportProvider.transport(), useShadowsocksTransport: false)
    }

    public func shadowSocksTransport() -> RESTTransport? {
        return selectTransport(transportProvider.shadowSocksTransport(), useShadowsocksTransport: true)
    }

    // MARK: -

    // MARK: Private API

    /// Selects a transport to use for sending an `URLRequest`
    ///
    /// This method returns the appropriate transport layer based on whether a tunnel is available, and whether it
    /// should be bypassed
    /// whenever a transport is requested.
    ///
    /// - Parameters:
    ///   - transport: The transport to use if there is no tunnel, or if it shouldn't be bypassed
    ///   - useShadowsocksTransport: A hint for enforcing a Shadowsocks transport when proxying a request via an
    /// available `Tunnel`
    /// - Returns: A transport to use for sending an `URLRequest`
    private func selectTransport(_ transport: RESTTransport?, useShadowsocksTransport: Bool) -> RESTTransport? {
        let tunnel = tunnelStore.getPersistentTunnels().first { tunnel in
            return tunnel.status == .connecting ||
                tunnel.status == .reasserting ||
                tunnel.status == .connected
        }

        if let tunnel, shouldByPassVPN(tunnel: tunnel) {
            return PacketTunnelTransport(
                tunnel: tunnel,
                useShadowsocksTransport: useShadowsocksTransport
            )
        }
        return transport
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
