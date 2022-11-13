//
//  MigrationFromUnversionedToV2.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-11-18.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadTypes

class MigrationFromUnversionedToV2: Migration {
    private let logger: Logger

    /// `TunnelSettings` stored data.
    private let settingsData: Data

    /// `DeviceState` stored data.
    private let deviceStateData: Data

    init(
        settingsData: Data,
        deviceStateData: Data,
        logger: Logger
    ) {
        self.settingsData = settingsData
        self.deviceStateData = deviceStateData
        self.logger = logger
    }

    func migrate(
        with middleware: SettingsStorageMiddleware,
        completion: @escaping (Error?) -> Void
    ) {
        do {
            // Tunnel settings
            logger.debug("Try to parse unversioned settings")

            let unversionedTunnelSettings = try middleware.parseUnversionedPayload(
                as: TunnelSettingsV2.self,
                from: settingsData
            )

            // Device state
            let unversionedDeviceState = try middleware.parseUnversionedPayload(
                as: DeviceState.self,
                from: deviceStateData
            )

            logger
                .debug(
                    "Store settings with version, current version: \(SchemaVersion.current.rawValue)"
                )

            // Save settings.
            try middleware.saveSettings(unversionedTunnelSettings)
            try middleware.saveDeviceState(unversionedDeviceState)

            completion(nil)
        } catch is DecodingError {
            completion(nil)
        } catch {
            logger.error(
                error: error,
                message: "Failed to migrate settings from unversioned, to new version."
            )

            completion(error)
        }
    }
}
