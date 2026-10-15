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
import MullvadLogging
import MullvadSettings
import MullvadTypes
import Operations

actor LoadTunnelConfigurationTask {
    private let logger = Logger(label: "LoadTunnelConfigurationOperation")
    private let interactor: TunnelInteractor
    private let settingsManager: SettingsManager

    init(interactor: TunnelInteractor, settingsManager: SettingsManager) {
        self.interactor = interactor
        self.settingsManager = settingsManager
    }

    func start() async {
        let settingsResult = readSettings()
        let deviceStateResult = readDeviceState()

        let tunnel = await interactor.getPersistentTunnel()
        let settings = settingsResult.flattenValue()
        let deviceState = deviceStateResult.flattenValue()

        await interactor.setSettings(settings ?? LatestTunnelSettings(), persist: false)
        await interactor.setDeviceState(deviceState ?? .loggedOut, persist: false)

        guard let tunnel else {
            await setTunnelAndLoadConfiguration(nil)
            return
        }

        if deviceState == nil {
            tunnel.removeFromPreferences { error in
                error.flatMap { self.logger.error(error: $0, message: "Failed to remove VPN configuration.") }
            }
            await setTunnelAndLoadConfiguration(nil)
        } else {
            await setTunnelAndLoadConfiguration(tunnel)
        }
    }

    private func setTunnelAndLoadConfiguration(_ tunnel: (any TunnelProtocol)?) async {
        await interactor.setTunnel(tunnel, shouldRefreshTunnelState: true)
        await interactor.setConfigurationLoaded()
    }

    private func readSettings() -> Result<LatestTunnelSettings?, Error> {
        Result { try settingsManager.readSettings() }
            .flatMapError { error in
                if let error = error as? ReadSettingsVersionError,
                    let keychainError = error.underlyingError as? KeychainError, keychainError == .itemNotFound
                {
                    logger.debug("Settings not found in keychain.")

                    return .success(nil)
                } else {
                    logger.error(
                        error: error,
                        message: "Cannot read settings."
                    )

                    return .failure(error)
                }
            }
    }

    private func readDeviceState() -> Result<DeviceState?, Error> {
        Result { try settingsManager.readDeviceState() }
            .flatMapError { error in
                if let error = error as? KeychainError, error == .itemNotFound {
                    logger.debug("Device state not found in keychain.")

                    return .success(nil)
                } else {
                    logger.error(
                        error: error,
                        message: "Cannot read device state."
                    )

                    return .failure(error)
                }
            }
    }
}
