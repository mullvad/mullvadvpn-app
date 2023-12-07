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
import MullvadTypes
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

        if let tunnel, shouldByPassVPN(tunnel: tunnel) {
            return PacketTunnelTransport(tunnel: tunnel)
        } else {
            return transportProvider.makeTransport()
        }
    }

    private func shouldByPassVPN(tunnel: any TunnelProtocol) -> Bool {
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
