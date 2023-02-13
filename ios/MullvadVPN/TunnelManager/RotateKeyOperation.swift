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

    private let rotationInterval: TimeInterval?
    private let logger = Logger(label: "ReplaceKeyOperation")

    init(
        dispatchQueue: DispatchQueue,
        interactor: TunnelInteractor,
        devicesProxy: REST.DevicesProxy,
        rotationInterval: TimeInterval?
    ) {
        self.interactor = interactor
        self.devicesProxy = devicesProxy
        self.rotationInterval = rotationInterval

        super.init(
            dispatchQueue: dispatchQueue,
            completionQueue: nil,
            completionHandler: nil
        )
    }

    override func main() {
        guard case let .loggedIn(accountData, deviceData) = interactor.deviceState else {
            finish(result: .failure(InvalidDeviceStateError()))
            return
        }

        if let rotationInterval = rotationInterval {
            let creationDate = deviceData.wgKeyData.creationDate
            let nextRotationDate = creationDate.addingTimeInterval(rotationInterval)

            if nextRotationDate > Date() {
                logger.debug("Throttle private key rotation.")

                finish(result: .success(false))
                return
            } else {
                logger.debug("Private key is old enough, rotate right away.")
            }
        } else {
            logger.debug("Rotate private key right away.")
        }

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
                deviceData.wgKeyData = StoredWgKeyData(
                    creationDate: Date(),
                    privateKey: newPrivateKey
                )

                interactor.setDeviceState(.loggedIn(accountData, deviceData), persist: true)

                finish(result: .success(true))
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
            finish(result: .failure(error))
        }
    }
}
