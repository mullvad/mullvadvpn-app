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
    func clear() throws
}

public class ShadowsocksLoader: ShadowsocksLoaderProtocol {
    let cache: ShadowsocksConfigurationCacheProtocol
    let relaySelector: ShadowsocksRelaySelectorProtocol
    let constraintsUpdater: RelayConstraintsUpdater

    private var relayConstraints = RelayConstraints()

    public init(
        cache: ShadowsocksConfigurationCacheProtocol,
        relaySelector: ShadowsocksRelaySelectorProtocol,
        constraintsUpdater: RelayConstraintsUpdater
    ) {
        self.cache = cache
        self.relaySelector = relaySelector
        self.constraintsUpdater = constraintsUpdater

        constraintsUpdater.onNewConstraints = { [weak self] newConstraints in
            self?.relayConstraints = newConstraints
            try? self?.cache.clear()
        }
    }

    public func clear() throws {
        try self.cache.clear()
    }

    /// Returns the last used shadowsocks configuration, otherwise a new randomized configuration.
    public func load() throws -> ShadowsocksConfiguration {
        do {
            // If a previous shadowsocks configuration was in cache, return it directly.
            return try cache.read()
        } catch {
            // There is no previous configuration either if this is the first time this code ran
            let newConfiguration = try create()
            try cache.write(newConfiguration)
            return newConfiguration
        }
    }

    /// Returns a randomly selected shadowsocks configuration.
    private func create() throws -> ShadowsocksConfiguration {
        let bridgeConfiguration = try relaySelector.getBridges()
        let closestRelay = try relaySelector.selectRelay(with: relayConstraints)

        guard let bridgeAddress = closestRelay?.ipv4AddrIn,
              let bridgeConfiguration else { throw POSIXError(.ENOENT) }

        return ShadowsocksConfiguration(
            address: .ipv4(bridgeAddress),
            port: bridgeConfiguration.port,
            password: bridgeConfiguration.password,
            cipher: bridgeConfiguration.cipher
        )
    }
}
