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

    private let keyRotationConfiguration: StoredWgKeyData.KeyRotationConfiguration
    private let logger = Logger(label: "ReplaceKeyOperation")

    init(
        dispatchQueue: DispatchQueue,
        interactor: TunnelInteractor,
        devicesProxy: REST.DevicesProxy,
        keyRotationConfiguration: StoredWgKeyData.KeyRotationConfiguration
    ) {
        self.interactor = interactor
        self.devicesProxy = devicesProxy
        self.keyRotationConfiguration = keyRotationConfiguration

        super.init(dispatchQueue: dispatchQueue, completionQueue: nil, completionHandler: nil)
    }

    override func main() {
        var deviceState = interactor.deviceState
        guard case .loggedIn(let accountData, var deviceData) = deviceState else {
            finish(result: .failure(InvalidDeviceStateError()))
            return
        }

        let nextRotationDate = deviceData.wgKeyData.getNextRotationDate(for: keyRotationConfiguration)
        if nextRotationDate > Date() {
            logger.debug("Throttle private key rotation.")

            finish(result: .success(false))
            return
        } else {
            logger.debug("Private key is old enough, rotate right away.")
        }

        deviceData.wgKeyData.markRotationAttempt()
        let newPrivateKey = deviceData.wgKeyData.getOrCreateNextPrivateKey()
        deviceState.updateData { _, storedDeviceData in
            storedDeviceData = deviceData
        }

        interactor.setDeviceState(deviceState, persist: true)

        logger.debug("Replacing old key with new key on server...")

        task = devicesProxy.rotateDeviceKey(
            accountNumber: accountData.number,
            identifier: deviceData.identifier,
            publicKey: newPrivateKey.publicKey,
            retryStrategy: .default
        ) { result in
            self.dispatchQueue.async {
                self.didRotateKey(newPrivateKey: newPrivateKey, result: result)
            }
        }
    }

    override func operationDidCancel() {
        task?.cancel()
        task = nil
    }

    private func didRotateKey(newPrivateKey: PrivateKey, result: Result<REST.Device, Error>) {
        switch result {
        case let .success(device):
            logger.debug("Successfully rotated device key. Persisting device state...")

            var deviceState = interactor.deviceState
            guard var deviceData = deviceState.deviceData else {
                finish(result: .failure(InvalidDeviceStateError()))
                return
            }

            deviceData.update(from: device)
            deviceData.wgKeyData = StoredWgKeyData(creationDate: Date(), privateKey: newPrivateKey)
            deviceState.updateData { _, storedDeviceData in
                storedDeviceData = deviceData
            }
            interactor.setDeviceState(deviceState, persist: true)

            if let tunnel = interactor.tunnel {
                _ = tunnel.notifyKeyRotation { [weak self] _ in
                    self?.finish(result: .success(true))
                }
            } else {
                finish(result: .success(true))
            }

        case let .failure(error):
            if !error.isOperationCancellationError {
                logger.error(error: error, message: "Failed to rotate device key.")
            }

            interactor.handleRestError(error)
            finish(result: .failure(error))
        }
    }
}
