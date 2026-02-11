//
//  NewAppVersionSystemNoticationHandler.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-02-11.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadSettings

final class NewAppVersionSystemNoticationHandler {
    private let appStoreMetaDataService: AppStoreMetaDataService
    private let settingsUpdater: SettingsUpdater
    private var tunnelSettings: LatestTunnelSettings
    private var observer: SettingsObserverBlock!

    init(
        appStoreMetaDataService: AppStoreMetaDataService,
        settingsUpdater: SettingsUpdater,
        tunnelSettings: LatestTunnelSettings
    ) {
        self.settingsUpdater = settingsUpdater
        self.appStoreMetaDataService = appStoreMetaDataService
        self.tunnelSettings = tunnelSettings

        self.observer = SettingsObserverBlock(
            didUpdateSettings: { [weak self] latestTunnelSettings in
                self?.tunnelSettings = latestTunnelSettings
            }
        )
        self.settingsUpdater.addObserver(observer)

        self.appStoreMetaDataService.onNewAppVersion = {
            if tunnelSettings.includeAllNetworks.includeAllNetworksIsEnabled {
                appStoreMetaDataService.sendSystemNotification()
            }
        }

        appStoreMetaDataService.scheduleTimer()
    }
}
