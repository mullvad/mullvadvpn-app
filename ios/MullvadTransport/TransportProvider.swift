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
    private let shadowsocksCache: ShadowsocksConfigurationCache

    public init(
        urlSessionTransport: URLSessionTransport,
        relayCache: RelayCache,
        addressCache: REST.AddressCache,
        shadowsocksCache: ShadowsocksConfigurationCache
    ) {
        self.urlSessionTransport = urlSessionTransport
        self.relayCache = relayCache
        self.addressCache = addressCache
        self.shadowsocksCache = shadowsocksCache
    }

    public func transport() -> RESTTransport? {
        urlSessionTransport
    }

    public func shadowsocksTransport() -> RESTTransport? {
        do {
            let shadowsocksConfiguration = try shadowsocksConfiguration()

            let shadowsocksURLSession = urlSessionTransport.urlSession
            let shadowsocksTransport = URLSessionShadowsocksTransport(
                urlSession: shadowsocksURLSession,
                shadowsocksConfiguration: shadowsocksConfiguration,
                addressCache: addressCache
            )

            return shadowsocksTransport
        } catch {
            logger.error(error: error, message: "Failed to produce shadowsocks configuration.")
            return nil
        }
    }

    /// Returns the last used shadowsocks configuration, otherwise a new randomized configuration.
    private func shadowsocksConfiguration() throws -> ShadowsocksConfiguration {
        // If a previous shadowsocks configuration was in cache, return it directly
        do {
            return try shadowsocksCache.read()
        } catch {
            logger.error(error: error, message: "Failed to read shadowsocks cache.")

            // There is no previous configuration either if this is the first time this code ran
            // Or because the previous shadowsocks configuration was invalid, therefore generate a new one.
            return try makeNewShadowsocksConfiguration()
        }
    }

    /// Returns a randomly selected shadowsocks configuration.
    private func makeNewShadowsocksConfiguration() throws -> ShadowsocksConfiguration {
        let cachedRelays = try relayCache.read()
        let bridgeAddress = RelaySelector.getShadowsocksRelay(relays: cachedRelays.relays)?.ipv4AddrIn
        let bridgeConfiguration = RelaySelector.getShadowsocksTCPBridge(relays: cachedRelays.relays)

        guard let bridgeAddress, let bridgeConfiguration else { throw POSIXError(.ENOENT) }

        let newConfiguration = ShadowsocksConfiguration(
            bridgeAddress: bridgeAddress,
            bridgePort: bridgeConfiguration.port,
            password: bridgeConfiguration.password,
            cipher: bridgeConfiguration.cipher
        )

        do {
            try shadowsocksCache.write(newConfiguration)
        } catch {
            logger.error(error: error, message: "Failed to persist shadowsocks cache.")
        }

        return newConfiguration
    }
}
