//
//  RelaySelectorWrapper.swift
//  PacketTunnel
//
//  Created by pronebird on 08/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes

public final class RelaySelectorWrapper: RelaySelectorProtocol {
    let relayCache: RelayCacheProtocol

    let tunnelSettingsUpdater: SettingsUpdater
    private var tunnelSettings = LatestTunnelSettings()
    private var observer: SettingsObserverBlock!

    deinit {
        self.tunnelSettingsUpdater.removeObserver(observer)
    }

    public init(
        relayCache: RelayCacheProtocol,
        tunnelSettingsUpdater: SettingsUpdater
    ) {
        self.relayCache = relayCache
        self.tunnelSettingsUpdater = tunnelSettingsUpdater

        self.addObserver()
    }

    public func selectRelays(
        connectionAttemptCount: UInt
    ) throws -> SelectedRelays {
        let relays = try relayCache.read().relays

        switch tunnelSettings.tunnelMultihopState {
        case .off:
            return try SinglehopPicker(
                constraints: tunnelSettings.relayConstraints,
                daitaSettings: tunnelSettings.daita,
                relays: relays,
                connectionAttemptCount: connectionAttemptCount
            ).pick()
        case .on:
            return try MultihopPicker(
                constraints: tunnelSettings.relayConstraints,
                daitaSettings: tunnelSettings.daita,
                relays: relays,
                connectionAttemptCount: connectionAttemptCount
            ).pick()
        }
    }

    private func addObserver() {
        self.observer = SettingsObserverBlock(didUpdateSettings: { [weak self] latestTunnelSettings in
            self?.tunnelSettings = latestTunnelSettings
        })

        tunnelSettingsUpdater.addObserver(observer)
    }
}
