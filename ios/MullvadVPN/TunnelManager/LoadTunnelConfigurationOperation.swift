//
//  LoadTunnelConfigurationOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 16/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadTypes
import Operations

class LoadTunnelConfigurationOperation: ResultOperation<Void, Error> {
    private let logger = Logger(label: "LoadTunnelConfigurationOperation")
    private let interactor: TunnelInteractor

    init(dispatchQueue: DispatchQueue, interactor: TunnelInteractor) {
        self.interactor = interactor

        super.init(dispatchQueue: dispatchQueue)
    }

    override func main() {
        let settingsResult = readSettings()
        let deviceStateResult = readDeviceState()

        let persistentTunnels = interactor.getPersistentTunnels()
        let tunnel = persistentTunnels.first
        let settings = settingsResult.flattenValue()
        let deviceState = deviceStateResult.flattenValue()

        interactor.setSettings(settings ?? TunnelSettingsV2(), persist: false)
        interactor.setDeviceState(deviceState ?? .loggedOut, persist: false)

        if let tunnel = tunnel, deviceState == nil {
            logger.debug("Remove orphaned VPN configuration.")

            tunnel.removeFromPreferences { error in
                if let error = error {
                    self.logger.error(
                        error: error,
                        message: "Failed to remove VPN configuration."
                    )
                }
                self.finishOperation(tunnel: nil)
            }
        } else {
            finishOperation(tunnel: tunnel)
        }
    }

    private func finishOperation(tunnel: Tunnel?) {
        interactor.setTunnel(tunnel, shouldRefreshTunnelState: true)
        interactor.setConfigurationLoaded()

        finish(completion: .success(()))
    }

    private func readSettings() -> Result<TunnelSettingsV2?, Error> {
        return Result { try SettingsManager.readSettings() }
            .flatMapError { error in
                if let error = error as? KeychainError, error == .itemNotFound {
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
        return Result { try SettingsManager.readDeviceState() }
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
