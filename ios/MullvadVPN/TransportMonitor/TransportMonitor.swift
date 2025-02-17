//
//  TransportMonitor.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-10-07.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadTypes

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

    /// Selects a transport to use for sending an `URLRequest`
    ///
    /// This method returns the appropriate transport layer based on whether a tunnel is available, and whether it should be bypassed whenever a transport is
    /// requested.
    ///
    /// - Returns: A transport to use for sending an `URLRequest`
    func makeTransport() -> RESTTransport? {
        let tunnel = tunnelStore.getPersistentTunnels().first { tunnel in
            tunnel.status == .connecting || tunnel.status == .reasserting || tunnel.status == .connected
        }

        if let tunnel, shouldBypassVPN(tunnel: tunnel) {
            return PacketTunnelTransport(tunnel: tunnel)
        } else {
            return transportProvider.makeTransport()
        }
    }

    private func shouldBypassVPN(tunnel: any TunnelProtocol) -> Bool {
        switch tunnel.status {
        case .connected:
            if case .error = tunnelManager.tunnelStatus.state {
                return true
            }
            return tunnelManager.isConfigurationLoaded && tunnelManager.deviceState == .revoked

        case .connecting, .reasserting:
            return true

        default:
            return false
        }
    }
}

final class APITransportMonitor: APITransportProviderProtocol {
    private let tunnelManager: TunnelManager
    private let tunnelStore: TunnelStore
    private let requestFactory: MullvadApiRequestFactory

    init(tunnelManager: TunnelManager, tunnelStore: TunnelStore, requestFactory: MullvadApiRequestFactory) {
        self.tunnelManager = tunnelManager
        self.tunnelStore = tunnelStore
        self.requestFactory = requestFactory
    }

    func makeTransport() -> APITransportProtocol? {
        let tunnel = tunnelStore.getPersistentTunnels().first { tunnel in
            tunnel.status == .connecting || tunnel.status == .reasserting || tunnel.status == .connected
        }

        return if let tunnel, shouldBypassVPN(tunnel: tunnel) {
            PacketTunnelAPITransport(tunnel: tunnel)
        } else {
            APITransport(requestFactory: requestFactory)
        }
    }

    private func shouldBypassVPN(tunnel: any TunnelProtocol) -> Bool {
        switch tunnel.status {
        case .connected:
            if case .error = tunnelManager.tunnelStatus.state {
                true
            } else {
                tunnelManager.isConfigurationLoaded && tunnelManager.deviceState == .revoked
            }

        case .connecting, .reasserting:
            true

        default:
            false
        }
    }
}
