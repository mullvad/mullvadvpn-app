//
//  TunnelTransportProvider.swift
//  PacketTunnel
//
//  Created by Marco Nikic on 2023-04-25.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging
import MullvadREST
import RelayCache
import RelaySelector

final class TunnelTransportProvider: RESTTransportProvider {
    private let urlSessionTransport: REST.URLSessionTransport
    private let relayCache: RelayCache
    private let logger = Logger(label: "TunnelTransportProvider")

    init(urlSessionTransport: REST.URLSessionTransport, relayCache: RelayCache) {
        self.urlSessionTransport = urlSessionTransport
        self.relayCache = relayCache
    }

    func transport() -> MullvadREST.RESTTransport? {
        urlSessionTransport
    }

    func shadowSocksTransport() -> MullvadREST.RESTTransport? {
        do {
            let cachedRelays = try relayCache.read()
            let shadowSocksConfiguration = RelaySelector.getShadowsocksTCPBridge(relays: cachedRelays.relays)
            let shadowSocksBridgeRelay = RelaySelector.getShadowSocksRelay(relays: cachedRelays.relays)

            guard let shadowSocksConfiguration = shadowSocksConfiguration,
                  let shadowSocksBridgeRelay = shadowSocksBridgeRelay
            else {
                logger.error("Could not get shadow socks bridge information.")
                return nil
            }

            let shadowSocksURLSession = urlSessionTransport.urlSession
            let shadowSocksTransport = REST.URLSessionShadowSocksTransport(
                urlSession: shadowSocksURLSession,
                shadowSocksConfiguration: shadowSocksConfiguration,
                shadowSocksBridgeRelay: shadowSocksBridgeRelay
            )

            return shadowSocksTransport
        } catch {
            logger.error(error: error)
        }
        return nil
    }
}
