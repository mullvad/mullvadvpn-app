//
//  TransportProvider.swift
//  MullvadTransport
//
//  Created by Marco Nikic on 2023-05-25.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging
import MullvadREST
import MullvadTypes
import RelayCache
import RelaySelector

public final class TransportProvider: RESTTransportProvider {
    private let urlSessionTransport: URLSessionTransport
    private let relayCache: RelayCache
    private let logger = Logger(label: "TransportProvider")
    private let transportStrategy = TransportStrategy()

    public init(urlSessionTransport: URLSessionTransport, relayCache: RelayCache) {
        self.urlSessionTransport = urlSessionTransport
        self.relayCache = relayCache
    }

    public func transport() -> RESTTransport? {
        urlSessionTransport
    }

    public func shadowSocksTransport() -> RESTTransport? {
        do {
            let cachedRelays = try relayCache.read()
            let shadowSocksConfiguration = RelaySelector.getShadowsocksTCPBridge(relays: cachedRelays.relays)
            let shadowSocksBridgeRelay = RelaySelector.getShadowSocksRelay(relays: cachedRelays.relays)

            guard let shadowSocksConfiguration,
                  let shadowSocksBridgeRelay
            else {
                logger.error("Could not get shadow socks bridge information.")
                return nil
            }

            let shadowSocksURLSession = urlSessionTransport.urlSession
            let shadowSocksTransport = URLSessionShadowSocksTransport(
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
