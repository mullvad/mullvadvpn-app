//
//  RotateKeyOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 15/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadSettings
import MullvadTypes
import Operations
import WireGuardKitTypes

class RotateKeyOperation: ResultOperation<Void> {
    private let logger = Logger(label: "RotateKeyOperation")
    private let interactor: TunnelInteractor
    private let devicesProxy: DeviceHandling
    private var task: Cancellable?

    init(dispatchQueue: DispatchQueue, interactor: TunnelInteractor, devicesProxy: DeviceHandling) {
        self.interactor = interactor
        self.devicesProxy = devicesProxy

        super.init(dispatchQueue: dispatchQueue, completionQueue: nil, completionHandler: nil)
    }

    override func main() {
        // Extract login metadata.
        guard case let .loggedIn(accountData, deviceData) = interactor.deviceState else {
            finish(result: .failure(InvalidDeviceStateError()))
            return
        }

        // Create key rotation.
        var keyRotation = WgKeyRotation(data: deviceData)

        // Check if key rotation can take place.
        guard keyRotation.shouldRotate else {
            logger.debug("Throttle private key rotation.")
            finish(result: .success(()))
            return
        }

        logger.debug("Private key is old enough, rotate right away.")

        // Mark the beginning of key rotation and receive the public key to push to backend.
        let publicKey = keyRotation.beginAttempt()

        // Persist mutated device data.
        interactor.setDeviceState(.loggedIn(accountData, keyRotation.data), persist: true)

        // Send REST request to rotate the device key.
        logger.debug("Replacing old key with new key on server...")

        task = devicesProxy.rotateDeviceKey(
            accountNumber: accountData.number,
            identifier: deviceData.identifier,
            publicKey: publicKey,
            retryStrategy: .default
        ) { [self] result in
            dispatchQueue.async { [self] in
                switch result {
                case let .success(device):
                    handleSuccess(accountData: accountData, fetchedDevice: device, keyRotation: keyRotation)
                case let .failure(error):
                    handleError(error)
                }
            }
        }
    }

    override func operationDidCancel() {
        task?.cancel()
        task = nil
    }

    private func handleSuccess(accountData: StoredAccountData, fetchedDevice: Device, keyRotation: WgKeyRotation) {
        logger.debug("Successfully rotated device key. Persisting device state...")

        var keyRotation = keyRotation

        // Mark key rotation completed.
        _ = keyRotation.setCompleted(with: fetchedDevice)

        // Persist changes.
        interactor.setDeviceState(.loggedIn(accountData, keyRotation.data), persist: true)

        // Notify the tunnel that key rotation took place and that it should reload VPN configuration.
        if let tunnel = interactor.tunnel {
            _ = tunnel.notifyKeyRotation { [weak self] _ in
                self?.finish(result: .success(()))
            }
        } else {
            finish(result: .success(()))
        }
    }

    private func handleError(_ error: Error) {
        if !error.isOperationCancellationError {
            logger.error(error: error, message: "Failed to rotate device key.")
        }

        interactor.handleRestError(error)
        finish(result: .failure(error))
    }
}
