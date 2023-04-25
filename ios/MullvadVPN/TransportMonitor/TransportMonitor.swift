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
import RelayCache

final class TransportMonitor: RESTTransportProvider {
    private let tunnelManager: TunnelManager
    private let tunnelStore: TunnelStore
    private let urlSessionTransport: REST.URLSessionTransport
    private let relayCacheTracker: RelayCacheTracker
    private let logger = Logger(label: "TransportMonitor")
    private var useAlternativeTransport = false

    private var currentTransport: RESTTransport?

    // MARK: -

    // MARK: Public API

    init(tunnelManager: TunnelManager, tunnelStore: TunnelStore, relayCacheTracker: RelayCacheTracker) {
        self.tunnelManager = tunnelManager
        self.tunnelStore = tunnelStore
        self.relayCacheTracker = relayCacheTracker

        urlSessionTransport = REST.URLSessionTransport(urlSession: REST.makeURLSession())
        currentTransport = urlSessionTransport
    }

    public func transport() -> MullvadREST.RESTTransport? {
        let tunnel = tunnelStore.getPersistentTunnels().first { tunnel in
            return tunnel.status == .connecting ||
                tunnel.status == .reasserting ||
                tunnel.status == .connected
        }

        if let tunnel = tunnel, shouldByPassVPN(tunnel: tunnel) {
            return PacketTunnelTransport(
                tunnel: tunnel,
                useAlternativeTransport: useAlternativeTransport
            )
        } else {
            return currentTransport
        }
    }

    public func selectNextTransport() {
        useAlternativeTransport = true
        currentTransport = shadowSocksTransport
    }

    // MARK: -

    // MARK: Private API

    /// The transport session that automatically rewrites the host and port of each `URLRequest` it creates to a locally
    /// hosted shadow socks proxy instance
    private var shadowSocksTransport: RESTTransport? {
        do {
            let cachedRelays = try relayCacheTracker.getCachedRelays()

            let shadowSocksConfiguration = cachedRelays.relays.bridge.shadowsocks.filter { $0.protocol == "tcp" }
                .randomElement()
            let shadowSocksBridgeRelay = cachedRelays.relays.bridge.relays.randomElement()

            guard let shadowSocksConfiguration = shadowSocksConfiguration,
                  let shadowSocksBridgeRelay = shadowSocksBridgeRelay
            else {
                logger.error("Could not get shadow socks bridge information.")
                return nil
            }

            let shadowSocksURLSession = urlSessionTransport.urlSession
            let transport = REST.URLSessionShadowSocksTransport(
                urlSession: shadowSocksURLSession,
                shadowSocksConfiguration: shadowSocksConfiguration,
                shadowSocksBridgeRelay: shadowSocksBridgeRelay
            )

            return transport
        } catch {
            logger.error(
                error: error,
                message: "Could not create shadow socks transport."
            )
            return nil
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
