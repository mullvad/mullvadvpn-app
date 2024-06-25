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
    let multihopUpdater: MultihopUpdater
    private var multihopState: MultihopState = .off
    private var observer: MultihopObserverBlock!

    deinit {
        self.multihopUpdater.removeObserver(observer)
    }

    private var relayConstraints = RelayConstraints()

    public init(
        cache: ShadowsocksConfigurationCacheProtocol,
        relaySelector: ShadowsocksRelaySelectorProtocol,
        constraintsUpdater: RelayConstraintsUpdater,
        multihopUpdater: MultihopUpdater
    ) {
        self.cache = cache
        self.relaySelector = relaySelector
        self.constraintsUpdater = constraintsUpdater
        self.multihopUpdater = multihopUpdater
        self.addObservers()
    }

    private func addObservers() {
        // The constraints gets updated a lot when observing the tunnel, clear the cache if the constraints have changed.
        constraintsUpdater.onNewConstraints = { [weak self] newConstraints in
            if self?.relayConstraints != newConstraints {
                self?.relayConstraints = newConstraints
                try? self?.clear()
            }
        }

        // The multihop state gets updated a lot when observing the tunnel, clear the cache if the multihop settings have changed.
        self.observer = MultihopObserverBlock(didUpdateMultihop: { [weak self] _, newMultihopState in
            if self?.multihopState != newMultihopState {
                self?.multihopState = newMultihopState
                try? self?.clear()
            }
        })
        multihopUpdater.addObserver(self.observer)
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
        let closestRelay = try relaySelector.selectRelay(with: relayConstraints, multihopState: multihopState)

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
