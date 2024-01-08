//
//  LocalShadowsocksLoader.swift
//  MullvadREST
//
//  Created by Mojgan on 2024-01-08.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

public protocol ShadowsocksLoaderProtocol {
    var onUpdatedConstraints: ((RelayConstraints) -> Void)? { get set }
    var configuration: ShadowsocksConfiguration { get throws }
}

public class ShadowsocksLoader: ShadowsocksLoaderProtocol {
    public var onUpdatedConstraints: ((RelayConstraints) -> Void)?
    public var configuration: ShadowsocksConfiguration {
        get throws {
            try load()
        }
    }

    private let shadowsocksCache: ShadowsocksConfigurationCache
    private let relayCache: RelayCacheProtocol
    private var relayConstraints = RelayConstraints()

    public init(shadowsocksCache: ShadowsocksConfigurationCache, relayCache: RelayCacheProtocol) {
        self.shadowsocksCache = shadowsocksCache
        self.relayCache = relayCache
        self.onUpdatedConstraints = { [weak self] value in
            self?.relayConstraints = value
        }
    }

    /// Returns the last used shadowsocks configuration, otherwise a new randomized configuration.
    private func load() throws -> ShadowsocksConfiguration {
        do {
            // If a previous shadowsocks configuration was in cache, return it directly.
            return try shadowsocksCache.read()
        } catch {
            // There is no previous configuration either if this is the first time this code ran
            // Or because the previous shadowsocks configuration was invalid, therefore generate a new one.
            let newConfiguration = try create()
            try shadowsocksCache.write(newConfiguration)
            return newConfiguration
        }
    }

    /// Returns a randomly selected shadowsocks configuration.
    private func create() throws -> ShadowsocksConfiguration {
        let cachedRelays = try relayCache.read()
        let bridgeConfiguration = RelaySelector.shadowsocksTCPBridge(from: cachedRelays.relays)
        let closestRelay = RelaySelector.closestShadowsocksRelayConstrained(
            by: relayConstraints,
            in: cachedRelays.relays
        )

        guard let bridgeAddress = closestRelay?.ipv4AddrIn, let bridgeConfiguration else { throw POSIXError(.ENOENT) }

        return ShadowsocksConfiguration(
            address: .ipv4(bridgeAddress),
            port: bridgeConfiguration.port,
            password: bridgeConfiguration.password,
            cipher: bridgeConfiguration.cipher
        )
    }
}
