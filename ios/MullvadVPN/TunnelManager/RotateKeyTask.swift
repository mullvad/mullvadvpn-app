//
//  RotateKeyTask.swift
//  MullvadVPN
//
//  Created by pronebird on 15/12/2021.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadSettings
import MullvadTypes
import Operations

struct RotateKeyResult: Sendable {
    let deviceState: DeviceState
}

final class RotateKeyTask: Sendable {

    private let logger = Logger(label: "RotateKeyOperation")

    private let initialDeviceState: DeviceState
    private let devicesProxy: DeviceHandling

    init(
        initialDeviceState: DeviceState,
        devicesProxy: DeviceHandling
    ) {
        self.initialDeviceState = initialDeviceState
        self.devicesProxy = devicesProxy
    }

    func run() async -> Result<RotateKeyResult, Error> {
        do {
            try Task.checkCancellation()

            guard case let .loggedIn(accountData, deviceData) = initialDeviceState else {
                return .failure(InvalidDeviceStateError())
            }

            var keyRotation = WgKeyRotation(data: deviceData)

            guard keyRotation.shouldRotate else {
                logger.debug("Throttle private key rotation.")
                return .success(
                    RotateKeyResult(
                        deviceState: initialDeviceState
                    )
                )
            }

            logger.debug("Private key is old enough, rotate right away.")

            let publicKey = keyRotation.beginAttempt()

            try Task.checkCancellation()

            let stateAfterAttempt = DeviceState.loggedIn(
                accountData,
                keyRotation.data
            )

            logger.debug("Replacing old key with new key on server...")

            let result = await devicesProxy.rotateDeviceKey(
                accountNumber: accountData.number,
                identifier: deviceData.identifier,
                publicKey: publicKey,
                retryStrategy: .default
            )

            try Task.checkCancellation()

            switch result {
            case let .success(device):
                return await handleSuccess(
                    accountData: accountData,
                    fetchedDevice: device,
                    keyRotation: keyRotation
                )

            case let .failure(error):
                _ = stateAfterAttempt
                return handleError(error)
            }
        } catch is CancellationError {
            return .failure(OperationError.cancelled)
        } catch {
            return .failure(error)
        }
    }

    private func handleSuccess(
        accountData: StoredAccountData,
        fetchedDevice: Device,
        keyRotation: WgKeyRotation
    ) async -> Result<RotateKeyResult, Error> {
        logger.debug(
            "Successfully rotated device key. Persisting device state..."
        )

        var keyRotation = keyRotation

        _ = keyRotation.setCompleted(with: fetchedDevice)

        let deviceState = DeviceState.loggedIn(
            accountData,
            keyRotation.data
        )

        try? Task.checkCancellation()

        return .success(
            RotateKeyResult(
                deviceState: deviceState
            )
        )
    }

    private func handleError(
        _ error: Error
    ) -> Result<RotateKeyResult, Error> {
        if !error.isOperationCancellationError {
            logger.error(
                error: error,
                message: "Failed to rotate device key."
            )
        }

        return .failure(error)
    }
}
