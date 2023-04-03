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

    private let rotationInterval: TimeInterval
    private let rotationFailureRetryInterval: TimeInterval
    private let logger = Logger(label: "ReplaceKeyOperation")

    init(
        dispatchQueue: DispatchQueue,
        interactor: TunnelInteractor,
        devicesProxy: REST.DevicesProxy,
        rotationInterval: TimeInterval,
        rotationFailureRetryInterval: TimeInterval
    ) {
        self.interactor = interactor
        self.devicesProxy = devicesProxy
        self.rotationInterval = rotationInterval
        self.rotationFailureRetryInterval = rotationFailureRetryInterval

        super.init(
            dispatchQueue: dispatchQueue,
            completionQueue: nil,
            completionHandler: nil
        )
    }

    override func main() {
        guard case var .loggedIn(accountData, deviceData) = interactor.deviceState else {
            finish(result: .failure(InvalidDeviceStateError()))
            return
        }

        let nextRotationDate: Date
        if let lastAttemptDate = deviceData.wgKeyData.lastRotationAttemptDate {
            nextRotationDate = lastAttemptDate.addingTimeInterval(self.rotationFailureRetryInterval)
        } else {
            nextRotationDate = deviceData.wgKeyData.creationDate.addingTimeInterval(self.rotationInterval)
        }

        if nextRotationDate > Date() {
            logger.debug("Throttle private key rotation.")

            finish(result: .success(false))
            return
        } else {
            logger.debug("Private key is old enough, rotate right away.")
        }

        deviceData.wgKeyData.lastRotationAttemptDate = Date()
        interactor.setDeviceState(.loggedIn(accountData, deviceData), persist: true)

        logger.debug("Replacing old key with new key on server...")

        let newPrivateKey = PrivateKey()

        task = devicesProxy.rotateDeviceKey(
            accountNumber: accountData.number,
            identifier: deviceData.identifier,
            publicKey: newPrivateKey.publicKey,
            retryStrategy: .default
        ) { result in
            self.dispatchQueue.async {
                self.didRotateKey(
                    newPrivateKey: newPrivateKey,
                    result: result
                )
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
            logger.debug("Successfully rotated device key. Persisting settings...")

            switch interactor.deviceState {
            case .loggedIn(let accountData, var deviceData):
                deviceData.update(from: device)

                let creationDate = Date()
                deviceData.wgKeyData = StoredWgKeyData(
                    creationDate: creationDate,
                    lastRotationAttemptDate: nil,
                    privateKey: newPrivateKey
                )

                interactor.setDeviceState(.loggedIn(accountData, deviceData), persist: true)

                if let tunnel = interactor.tunnel {
                    _ = tunnel.notifyKeyRotation { [weak self] _ in
                        self?.finish(result: .success(true))
                    }
                } else {
                    finish(result: .success(true))
                }
            default:
                finish(result: .failure(InvalidDeviceStateError()))
            }

        case let .failure(error):
            if !error.isOperationCancellationError {
                logger.error(
                    error: error,
                    message: "Failed to rotate device key."
                )
            }

            switch interactor.deviceState {
            case .loggedIn(let accountData, var deviceData):
                deviceData.wgKeyData.lastRotationAttemptDate = Date()
                interactor.setDeviceState(.loggedIn(accountData, deviceData), persist: true)

            default:
                finish(result: .failure(InvalidDeviceStateError()))
            }

            finish(result: .failure(error))
        }
    }
}
