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
    let settingsUpdater: SettingsUpdater

    private var observer: SettingsObserverBlock!
    private var tunnelSettings = LatestTunnelSettings()
    private let settingsStrategy = TunnelSettingsStrategy()

    deinit {
        self.settingsUpdater.removeObserver(observer)
    }

    public init(
        cache: ShadowsocksConfigurationCacheProtocol,
        relaySelector: ShadowsocksRelaySelectorProtocol,
        settingsUpdater: SettingsUpdater
    ) {
        self.cache = cache
        self.relaySelector = relaySelector
        self.settingsUpdater = settingsUpdater
        self.addObservers()
    }

    private func addObservers() {
        observer =
            SettingsObserverBlock(
                didUpdateSettings: { [weak self] _, latestTunnelSettings in
                    guard let self else { return }
                    if settingsStrategy.shouldReconnectToNewRelay(
                        oldSettings: tunnelSettings,
                        newSettings: latestTunnelSettings
                    ) {
                        try? clear()
                    }
                    tunnelSettings = latestTunnelSettings
                }
            )
        settingsUpdater.addObserver(self.observer)
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
        let closestRelay = try relaySelector.selectRelay(with: tunnelSettings)

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
