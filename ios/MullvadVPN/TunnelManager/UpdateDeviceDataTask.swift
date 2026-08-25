//
//  UpdateDeviceDataTask.swift
//  MullvadVPN
//
//  Created by pronebird on 13/05/2022.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadSettings
import MullvadTypes

final class UpdateDeviceDataTask: Sendable {

    private let initialDeviceState: DeviceState
    private let devicesProxy: DeviceHandling

    init(
        initialDeviceState: DeviceState,
        devicesProxy: DeviceHandling
    ) {
        self.initialDeviceState = initialDeviceState
        self.devicesProxy = devicesProxy
    }

    func run() async -> Result<DeviceState, Error> {
        do {
            try Task.checkCancellation()

            guard case let .loggedIn(accountData, deviceData) = initialDeviceState else {
                return .failure(InvalidDeviceStateError())
            }

            let result = await devicesProxy.getDevice(
                accountNumber: accountData.number,
                identifier: deviceData.identifier,
                retryStrategy: .default
            )

            try Task.checkCancellation()

            switch result {
            case let .success(device):
                var storedDevice = deviceData
                storedDevice.update(from: device)

                return .success(
                    .loggedIn(
                        accountData,
                        storedDevice
                    )
                )

            case let .failure(error):
                return .failure(error)
            }
        } catch is CancellationError {
            return .failure(CancellationError())
        } catch {
            return .failure(error)
        }
    }
}
