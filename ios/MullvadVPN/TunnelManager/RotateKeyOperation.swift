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
import MullvadREST
import MullvadSettings
import MullvadTypes
import Operations

class RotateKeyOperation: ResultOperation<Void>, @unchecked Sendable {
    private let logger = Logger(label: "RotateKeyOperation")
    private let devicesProxy: DeviceHandling
    private let deviceState: @Sendable () -> DeviceState
    private let onUpdateAccount: @Sendable (DeviceState) -> Void
    private var task: Cancellable?

    init(
        dispatchQueue: DispatchQueue,
        devicesProxy: DeviceHandling,
        deviceState: @escaping @Sendable () -> DeviceState,
        onUpdateAccount: @escaping @Sendable (DeviceState) -> Void
    ) {
        self.devicesProxy = devicesProxy
        self.deviceState = deviceState
        self.onUpdateAccount = onUpdateAccount
        super.init(dispatchQueue: dispatchQueue, completionQueue: nil, completionHandler: nil)
    }

    override func main() {
        // Extract login metadata.
        guard case let .loggedIn(accountData, deviceData) = deviceState() else {
            finish(result: .failure(InvalidDeviceStateError()))
            return
        }

        // Create key rotation.
        nonisolated(unsafe) var keyRotation = WgKeyRotation(data: deviceData)

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
        onUpdateAccount(.loggedIn(accountData, keyRotation.data))

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
        onUpdateAccount(.loggedIn(accountData, keyRotation.data))

        finish(result: .success(()))
    }

    private func handleError(_ error: Error) {
        if !error.isOperationCancellationError {
            logger.error(error: error, message: "Failed to rotate device key.")
        }
        finish(result: .failure(error))
    }
}
