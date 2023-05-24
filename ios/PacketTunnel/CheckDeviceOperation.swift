//
//  CheckDeviceOperation.swift
//  PacketTunnel
//
//  Created by pronebird on 20/04/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadTypes
import Operations
import class WireGuardKit.PrivateKey
import class WireGuardKit.PublicKey

struct CheckDeviceResult {
    var deviceCheck: DeviceCheck?
    var isKeyRotated = false
    var error: Error?
}

final class CheckDeviceOperation: AsyncOperation {
    typealias CompletionHandler = (CheckDeviceResult) -> Void

    private let logger = Logger(label: "CheckDeviceOperation")

    private let accountsProxy: REST.AccountsProxy
    private let devicesProxy: REST.DevicesProxy
    private let shouldImmediatelyRotateKeyOnMismatch: Bool
    private let completionHandler: CompletionHandler

    private var tasks: [Cancellable] = []

    init(
        dispatchQueue: DispatchQueue,
        accountsProxy: REST.AccountsProxy,
        devicesProxy: REST.DevicesProxy,
        shouldImmediatelyRotateKeyOnMismatch: Bool,
        completionHandler: @escaping CompletionHandler
    ) {
        self.accountsProxy = accountsProxy
        self.devicesProxy = devicesProxy
        self.shouldImmediatelyRotateKeyOnMismatch = shouldImmediatelyRotateKeyOnMismatch
        self.completionHandler = completionHandler

        super.init(dispatchQueue: dispatchQueue)
    }

    override func main() {
        startCheck { checkResult in
            self.completionHandler(checkResult)

            self.finish(error: checkResult.error)
        }
    }

    override func operationDidCancel() {
        accountTask?.cancel()
        deviceTask?.cancel()
        rotationTask?.cancel()
    }

    // MARK: - Flow

    private func startCheck(completion: @escaping (CheckDeviceResult) -> Void) {
        var checkResult = CheckDeviceResult()

        do {
            let deviceState = try SettingsManager.readDeviceState()
            guard case let .loggedIn(accountData, deviceData) = deviceState else {
                throw InvalidDeviceStateError()
            }

            fetchData(accountNumber: accountData.number, deviceIdentifier: deviceData.identifier) { result in
                checkResult.deviceCheck = result.value
                checkResult.error = result.error

                // Attempt to rotate the key when key mismatch is detected.
                if checkResult.deviceCheck?.isKeyMismatch == true {
                    self.maybeRotateKey { isKeyRotated, error in
                        if isKeyRotated {
                            checkResult.deviceCheck?.isKeyMismatch = false
                        }
                        checkResult.deviceCheck?.lastKeyRotationAttemptDate = Date()
                        checkResult.isKeyRotated = isKeyRotated
                        checkResult.error = error
                        completion(checkResult)
                    }
                } else {
                    completion(checkResult)
                }
            }
        } catch {
            checkResult.error = error
            completion(checkResult)
        }
    }

    private func fetchData(
        accountNumber: String,
        deviceIdentifier: String,
        completion: @escaping (Result<DeviceCheck, Error>) -> Void
    ) {
        var accountResult: Result<REST.AccountData, Error>?
        var deviceResult: Result<REST.Device, Error>?

        let dispatchGroup = DispatchGroup()

        dispatchGroup.enter()
        let accountTask = accountsProxy.getAccountData(accountNumber: accountNumber, retryStrategy: .noRetry) { result in
            accountResult = result
            dispatchGroup.leave()
        }

        dispatchGroup.enter()
        let deviceTask = devicesProxy.getDevice(
            accountNumber: accountNumber,
            identifier: deviceIdentifier,
            retryStrategy: .noRetry
        ) { result in
            deviceResult = result
            dispatchGroup.leave()
        }

        tasks.append(contentsOf: [accountTask, deviceTask])

        dispatchGroup.notify(queue: dispatchQueue) {
            guard let accountResult = accountResult, let deviceResult = deviceResult else { return }

            let result = Result {
                try self.mapResultsToDeviceCheck(accountResult: accountResult, deviceResult: deviceResult)
            }

            completion(result)
        }
    }

    private func maybeRotateKey(completion: @escaping (Bool, Error?) -> Void) {
        do {
            var deviceState = try SettingsManager.readDeviceState()
            guard case .loggedIn(let accountData, var deviceData) = deviceState else { throw InvalidDeviceStateError() }

            guard deviceData.wgKeyData.shouldPacketTunnelRotateTheKey(
                shouldRotateImmediately: shouldImmediatelyRotateKeyOnMismatch
            ) else {
                completion(false, nil)
                return
            }

            logger.debug("Rotate private key from packet tunnel.")

            deviceData.wgKeyData.markRotationAttempt()
            let nextPrivateKey = deviceData.wgKeyData.getOrCreateNextPrivateKey()

            deviceState.updateData { _, storedDeviceData in
                storedDeviceData = deviceData
            }

            try SettingsManager.writeDeviceState(deviceState)

            rotationTask = devicesProxy.rotateDeviceKey(
                accountNumber: accountData.number,
                identifier: deviceData.identifier,
                publicKey: nextPrivateKey.publicKey,
                retryStrategy: .default
            ) { result in
                self.dispatchQueue.async {
                    self.handleRotationResult(result, nextPrivateKey: nextPrivateKey, completion: completion)
                }
            }
        } catch {
            completion(false, error)
        }
    }

    private func handleRotationResult(
        _ result: Result<REST.Device, Error>,
        nextPrivateKey: PrivateKey,
        completion: @escaping (Bool, Error?) -> Void
    ) {
        switch result {
        case let .success(device):
            self.logger.debug("Successfully rotated device key. Persisting device state...")

            do {
                try self.saveNewPrivateKey(nextPrivateKey, device: device)

                completion(true, nil)
            } catch {
                self.logger.error(error: error, message: "Failed to persist device state.")

                completion(false, error)
            }

        case let .failure(error):
            if !error.isOperationCancellationError {
                self.logger.error(error: error, message: "Failed to rotate device key.")
            }

            completion(false, error)
        }
    }

    // MARK: - Private helpers

    private func mapResultsToDeviceCheck(
        accountResult: Result<REST.AccountData, Error>,
        deviceResult: Result<REST.Device, Error>
    ) throws -> DeviceCheck {
        let deviceState = try SettingsManager.readDeviceState()
        guard let deviceData = deviceState.deviceData else { throw InvalidDeviceStateError() }

        var deviceCheck = DeviceCheck()

        switch accountResult {
        case let .success(serverAccountData):
            deviceCheck.accountExpiry = serverAccountData.expiry
            deviceCheck.isAccountInvalid = false

        case let .failure(error):
            if let error = error as? REST.Error, error.compareErrorCode(.invalidAccount) {
                deviceCheck.isAccountInvalid = true
            }

            if !error.isOperationCancellationError {
                logger.error(error: error, message: "Failed to fetch account data.")
            }
        }

        switch deviceResult {
        case let .success(serverDevice):
            deviceCheck.isDeviceRevoked = false
            deviceCheck.isKeyMismatch = serverDevice.pubkey != deviceData.wgKeyData.privateKey.publicKey

        case let .failure(error):
            if let error = error as? REST.Error, error.compareErrorCode(.deviceNotFound) {
                deviceCheck.isDeviceRevoked = true
            }

            if !error.isOperationCancellationError {
                logger.error(error: error, message: "Failed to fetch device data.")
            }
        }

        return deviceCheck
    }

    private func saveNewPrivateKey(_ newPrivateKey: PrivateKey, device: REST.Device) throws {
        var deviceState = try SettingsManager.readDeviceState()

        guard var deviceData = deviceState.deviceData else { throw InvalidDeviceStateError() }

        deviceData.wgKeyData = StoredWgKeyData(creationDate: Date(), privateKey: newPrivateKey)
        deviceData.update(from: device)

        deviceState.updateData { _, storedDeviceData in
            storedDeviceData = deviceData
        }

        try SettingsManager.writeDeviceState(deviceState)
    }
}

private struct InvalidDeviceStateError: LocalizedError {
    var errorDescription: String? {
        return "Cannot complete device check because device is no longer logged in."
    }
}
