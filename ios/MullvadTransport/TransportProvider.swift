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
    private let addressCache: REST.AddressCache

    public init(urlSessionTransport: URLSessionTransport, relayCache: RelayCache, addressCache: REST.AddressCache) {
        self.urlSessionTransport = urlSessionTransport
        self.relayCache = relayCache
        self.addressCache = addressCache
    }

    public func transport() -> RESTTransport? {
        urlSessionTransport
    }

    public func shadowsocksTransport() -> RESTTransport? {
        do {
            let cachedRelays = try relayCache.read()
            let shadowsocksConfiguration = RelaySelector.getShadowsocksTCPBridge(relays: cachedRelays.relays)
            let shadowsocksBridgeRelay = RelaySelector.getShadowsocksRelay(relays: cachedRelays.relays)

            guard let shadowsocksConfiguration,
                  let shadowsocksBridgeRelay
            else {
                logger.error("Could not get shadow socks bridge information.")
                return nil
            }

            let shadowsocksURLSession = urlSessionTransport.urlSession
            let shadowsocksTransport = URLSessionShadowsocksTransport(
                urlSession: shadowsocksURLSession,
                shadowsocksConfiguration: shadowsocksConfiguration,
                shadowsocksBridgeRelay: shadowsocksBridgeRelay,
                addressCache: addressCache
            )

            return shadowsocksTransport
        } catch {
            logger.error(error: error)
        }
        return nil
    }
}
