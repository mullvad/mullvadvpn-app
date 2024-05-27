//
//  LocalShadowsocksLoader.swift
//  MullvadREST
//
//  Created by Mojgan on 2024-01-08.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import MullvadTypes

public protocol ShadowsocksLoaderProtocol {
    func load() throws -> ShadowsocksConfiguration
    func reloadConfiguration() throws
}

public class ShadowsocksLoader: ShadowsocksLoaderProtocol {
    let shadowsocksCache: ShadowsocksConfigurationCache
    let shadowsocksRelaySelector: ShadowsocksRelaySelectorProtocol
    let constraintsUpdater: RelayConstraintsUpdater

    private var relayConstraints = RelayConstraints()

    public init(
        shadowsocksCache: ShadowsocksConfigurationCache,
        shadowsocksRelaySelector: ShadowsocksRelaySelectorProtocol,
        constraintsUpdater: RelayConstraintsUpdater
    ) {
        self.shadowsocksCache = shadowsocksCache
        self.shadowsocksRelaySelector = shadowsocksRelaySelector
        self.constraintsUpdater = constraintsUpdater

        constraintsUpdater.onNewConstraints = { [weak self] newConstraints in
            try? self?.invalidate()
            self?.relayConstraints = newConstraints
        }
    }

    public func reloadConfiguration() throws {
        let newConfiguration = try create()
        try shadowsocksCache.write(newConfiguration)
    }

    /// Returns the last used shadowsocks configuration, otherwise a new randomized configuration.
    public func load() throws -> ShadowsocksConfiguration {
        do {
            // If a previous shadowsocks configuration was in cache, return it directly.
            return try shadowsocksCache.read()
        } catch {
            // There is no previous configuration either if this is the first time this code ran
            let newConfiguration = try create()
            try shadowsocksCache.write(newConfiguration)
            return newConfiguration
        }
    }

    /// Returns a randomly selected shadowsocks configuration.
    private func create() throws -> ShadowsocksConfiguration {
        let bridgeConfiguration = try shadowsocksRelaySelector.getBridges()
        let closestRelay = try shadowsocksRelaySelector.selectRelay(with: relayConstraints)

        guard let bridgeAddress = closestRelay?.ipv4AddrIn,
              let bridgeConfiguration else { throw POSIXError(.ENOENT) }

        return ShadowsocksConfiguration(
            address: .ipv4(bridgeAddress),
            port: bridgeConfiguration.port,
            password: bridgeConfiguration.password,
            cipher: bridgeConfiguration.cipher
        )
    }

    private func invalidate() throws {
        // because the previous shadowsocks configuration was invalid, therefore generate a new one.
        let newConfiguration = try create()
        try shadowsocksCache.write(newConfiguration)
    }
}
