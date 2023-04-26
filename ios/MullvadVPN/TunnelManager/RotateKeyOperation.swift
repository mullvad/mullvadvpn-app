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
import MullvadTypes
import Operations
import class WireGuardKitTypes.PrivateKey

class RotateKeyOperation: ResultOperation<Bool> {
    private let interactor: TunnelInteractor

    private let devicesProxy: REST.DevicesProxy
    private var task: Cancellable?

    private let logger = Logger(label: "ReplaceKeyOperation")

    init(dispatchQueue: DispatchQueue, interactor: TunnelInteractor, devicesProxy: REST.DevicesProxy) {
        self.interactor = interactor
        self.devicesProxy = devicesProxy

        super.init(dispatchQueue: dispatchQueue, completionQueue: nil, completionHandler: nil)
    }

    override func main() {
        // Extract login metadata
        guard case let .loggedIn(accountData, deviceData) = interactor.deviceState else {
            finish(result: .failure(InvalidDeviceStateError()))
            return
        }

        // Create key rotation.
        var keyRotation = WgKeyRotation(data: deviceData)

        // Check if key rotation can take place.
        guard keyRotation.shouldRotateTheKey else {
            logger.debug("Throttle private key rotation.")
            finish(result: .success(false))
            return
        }

        // Mark rotation attempt and persist it.
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
            self.dispatchQueue.async {
                switch result {
                case let .success(device):
                    self.handleSuccess(with: device)
                case let .failure(error):
                    self.handleError(error)
                }
            }
        }
    }

    override func operationDidCancel() {
        task?.cancel()
        task = nil
    }

    private func handleSuccess(with device: Device) {
        logger.debug("Successfully rotated device key. Persisting device state...")

        // Fetch login metadata once again.
        guard case let .loggedIn(accountData, deviceData) = interactor.deviceState else {
            finish(result: .failure(InvalidDeviceStateError()))
            return
        }

        // Re-create key rotation with re-fetched data.
        var keyRotation = WgKeyRotation(data: deviceData)

        // Mark key rotation completed.
        keyRotation.setCompleted(with: device)

        // Persist changes.
        interactor.setDeviceState(.loggedIn(accountData, deviceData), persist: true)

        // Notify the tunnel that key rotation took place and that it should reload VPN configuration.
        if let tunnel = interactor.tunnel {
            _ = tunnel.notifyKeyRotation { [weak self] _ in
                self?.finish(result: .success(true))
            }
        } else {
            finish(result: .success(true))
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
