//
//  LocalShadowsocksLoader.swift
//  MullvadREST
//
//  Created by Mojgan on 2024-01-08.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import MullvadTypes

public final class ShadowsocksLoader: ShadowsocksLoaderProtocol, SwiftShadowsocksBridgeProviding, Sendable {
    let cache: ShadowsocksConfigurationCacheProtocol
    let relaySelector: ShadowsocksRelaySelectorProtocol
    let settingsUpdater: SettingsUpdater

    nonisolated(unsafe) private var observer: SettingsObserverBlock!
    nonisolated(unsafe) private var tunnelSettings = LatestTunnelSettings()
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
                didUpdateSettings: { [weak self] latestTunnelSettings in
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

    public func bridge() -> ShadowsocksConfiguration? {
        try? load()
    }
}
