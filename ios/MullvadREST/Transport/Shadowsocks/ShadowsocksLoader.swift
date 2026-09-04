// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadRustRuntime
import MullvadSettings
import MullvadTypes

public final class ShadowsocksLoader: ShadowsocksLoaderProtocol, ShadowsocksBridgeProvider, Sendable {
    let cache: ShadowsocksConfigurationCacheProtocol
    let relaySelector: ShadowsocksRelaySelectorProtocol
    let settingsUpdater: SettingsUpdater

    nonisolated(unsafe) private var observer: SettingsObserverBlock!
    nonisolated(unsafe) private var tunnelSettings: LatestTunnelSettings
    private let settingsStrategy = TunnelSettingsStrategy()

    deinit {
        self.settingsUpdater.removeObserver(observer)
    }

    public init(
        cache: ShadowsocksConfigurationCacheProtocol,
        relaySelector: ShadowsocksRelaySelectorProtocol,
        tunnelSettings: LatestTunnelSettings,
        settingsUpdater: SettingsUpdater
    ) {
        self.cache = cache
        self.relaySelector = relaySelector
        self.tunnelSettings = tunnelSettings
        self.settingsUpdater = settingsUpdater
        self.addObservers()
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

    public func bridge() -> ShadowsocksWrapper? {
        guard let data = try? load() else { return nil }
        return newShadowsocksAccessMethodSetting(
            address: data.address.rawValue,
            port: data.port,
            password: data.password,
            cipher: data.cipher)
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

    /// Returns a randomly selected shadowsocks configuration.
    private func create() throws -> ShadowsocksConfiguration {
        let bridgeConfiguration = try relaySelector.getBridgeConfig()
        let closestBridge = try relaySelector.selectBridge(with: tunnelSettings)

        guard let bridgeAddress = closestBridge?.ipv4AddrIn,
            let bridgeConfiguration
        else { throw POSIXError(.ENOENT) }

        return ShadowsocksConfiguration(
            address: .ipv4(bridgeAddress),
            port: bridgeConfiguration.port,
            password: bridgeConfiguration.password,
            cipher: bridgeConfiguration.cipher
        )
    }
}
