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
    private let relayCacheTracker: RelayCacheTracker
    
    // Shadow Socks Proxies
    
    private var proxies: [HttpProxy] = []
    

    init(tunnelManager: TunnelManager, tunnelStore: TunnelStore, relayCacheTracker: RelayCacheTracker) {
        self.tunnelManager = tunnelManager
        self.tunnelStore = tunnelStore
        self.relayCacheTracker = relayCacheTracker

        urlSessionTransport = REST.URLSessionTransport(urlSession: REST.makeURLSession())
    }
    
    var shadowSocksTransport: RESTTransport? {
        // Create a new shadow socks proxy for each request
        
        let cachedRelays = try! relayCacheTracker.getCachedRelays()
        let shadowSocksRelay = cachedRelays.relays.bridge.shadowsocks.filter { $0.protocol == "tcp" }.randomElement()!
        let shadowSocksBridge = cachedRelays.relays.bridge.relays.randomElement()!

        print("Will try to connect to \(shadowSocksBridge.ipv4AddrIn):\(shadowSocksRelay.port) password: \(shadowSocksRelay.password) and cipher: \(shadowSocksRelay.cipher)")
        
        let shadowSocksProxy = HttpProxy(remoteAddress: shadowSocksBridge.ipv4AddrIn,
                                     remotePort: shadowSocksRelay.port,
                                     password: shadowSocksRelay.password,
                                     cipher: shadowSocksRelay.cipher)
        
        shadowSocksProxy.start()
        proxies.append(shadowSocksProxy)
        
        DispatchQueue.global().asyncAfter(deadline: .now() + 10) { [weak self] in
            shadowSocksProxy.stop()
            self?.proxies.removeAll { storedProxy in
                storedProxy.uuid == shadowSocksProxy.uuid
            }
        }
        let shadowSocksURLSession = REST.makeURLSession(httpProxyConfiguration: REST.HTTPProxyConfiguration(port: shadowSocksProxy.localPort()))

        let transport = REST.URLSessionTransport(urlSession: shadowSocksURLSession)
        
        return transport
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
