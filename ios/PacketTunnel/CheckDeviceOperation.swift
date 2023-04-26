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
import class WireGuardKitTypes.PrivateKey
import class WireGuardKitTypes.PublicKey

final class CheckDeviceOperation: ResultOperation<DeviceCheck> {
    private let logger = Logger(label: "CheckDeviceOperation")

    private let accountsProxy: REST.AccountsProxy
    private let devicesProxy: REST.DevicesProxy
    private let shouldImmediatelyRotateKeyOnMismatch: Bool

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

        super.init(dispatchQueue: dispatchQueue, completionQueue: dispatchQueue, completionHandler: completionHandler)
    }

    override func main() {
        startFlow { result in
            self.finish(result: result)
        }
    }

    override func operationDidCancel() {
        tasks.forEach { $0.cancel() }
    }

    // MARK: - Flow

    private func startFlow(completion: @escaping (Result<DeviceCheck, Error>) -> Void) {
        do {
            guard case let .loggedIn(accountData, deviceData) = try SettingsManager.readDeviceState() else {
                throw InvalidDeviceStateError()
            }

            fetchData(
                accountNumber: accountData.number,
                deviceIdentifier: deviceData.identifier
            ) { [self] accountResult, deviceResult in
                didReceiveData(accountResult: accountResult, deviceResult: deviceResult, completion: completion)
            }
        } catch {
            completion(.failure(error))
        }
    }

    private func didReceiveData(
        accountResult: Result<AccountData, Error>,
        deviceResult: Result<Device, Error>,
        completion: @escaping (Result<DeviceCheck, Error>) -> Void
    ) {
        do {
            let accountVerdict = try accountVerdict(from: accountResult)
            let deviceVerdict = try deviceVerdict(from: deviceResult)

            if deviceVerdict == .keyMismatch {
                rotateKeyIfNeeded { rotationResult in
                    completion(rotationResult.map { rotationStatus in
                        return DeviceCheck(
                            accountVerdict: accountVerdict,
                            deviceVerdict: rotationStatus.isSucceeded ? .good : .keyMismatch,
                            keyRotationStatus: rotationStatus
                        )
                    })
                }
            } else {
                completion(.success(DeviceCheck(
                    accountVerdict: accountVerdict,
                    deviceVerdict: deviceVerdict,
                    keyRotationStatus: .none
                )))
            }
        } catch {
            completion(.failure(error))
        }
    }

    // MARK: - Data fetch

    private func fetchData(
        accountNumber: String, deviceIdentifier: String,
        completion: @escaping (Result<AccountData, Error>, Result<Device, Error>) -> Void
    ) {
        var accountResult: Result<AccountData, Error> = .failure(OperationError.cancelled)
        var deviceResult: Result<Device, Error> = .failure(OperationError.cancelled)

        let dispatchGroup = DispatchGroup()

        dispatchGroup.enter()
        let accountTask = accountsProxy
            .getAccountData(accountNumber: accountNumber, retryStrategy: .noRetry) { result in
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
            completion(accountResult, deviceResult)
        }
    }

    // MARK: - Key rotation

    private func rotateKeyIfNeeded(completion: @escaping (Result<KeyRotationStatus, Error>) -> Void) {
        let deviceState: DeviceState
        do {
            deviceState = try SettingsManager.readDeviceState()
        } catch {
            logger.error(error: error, message: "Failed to read device state before rotating the key.")
            completion(.failure(error))
            return
        }

        guard case let .loggedIn(accountData, deviceData) = deviceState else {
            logger.debug("Will not attempt to rotate the key as device is no longer logged in.")
            completion(.failure(InvalidDeviceStateError()))
            return
        }

        var keyRotation = WgKeyRotation(data: deviceData)
        guard keyRotation.shouldPacketTunnelRotateTheKey(shouldRotateImmediately: shouldImmediatelyRotateKeyOnMismatch)
        else {
            completion(.success(.none))
            return
        }

        let publicKey = keyRotation.beginAttempt()

        do {
            try SettingsManager.writeDeviceState(.loggedIn(accountData, keyRotation.data))
        } catch {
            logger.error(error: error, message: "Failed to persist updated device state before rotating the key.")
            completion(.failure(error))
            return
        }

        logger.debug("Rotate private key from packet tunnel.")

        let task = devicesProxy.rotateDeviceKey(
            accountNumber: accountData.number,
            identifier: deviceData.identifier,
            publicKey: publicKey,
            retryStrategy: .default
        ) { result in
            self.dispatchQueue.async {
                let returnResult = result.tryMap { device -> KeyRotationStatus in
                    try self.completeKeyRotation(device)
                    return .succeeded(Date())
                }
                .flatMapError { error in
                    self.logger.error(error: error, message: "Failed to rotate device key.")

                    if error.isOperationCancellationError {
                        return .failure(error)
                    } else {
                        return .success(.attempted(Date()))
                    }
                }

                completion(returnResult)
            }
        }

        tasks.append(task)
    }

    private func completeKeyRotation(_ device: Device) throws {
        logger.debug("Successfully rotated device key. Persisting device state...")

        let deviceState = try SettingsManager.readDeviceState()
        guard case let .loggedIn(accountData, deviceData) = deviceState else {
            logger.debug("Will not persist device state after rotating the key because device is no longer logged in.")
            throw InvalidDeviceStateError()
        }

        var keyRotation = WgKeyRotation(data: deviceData)
        keyRotation.setCompleted(with: device)

        do {
            try SettingsManager.writeDeviceState(.loggedIn(accountData, keyRotation.data))
        } catch {
            logger.error(error: error, message: "Failed to persist device state after rotating the key.")
            throw error
        }
    }

    // MARK: - Private helpers

    private func accountVerdict(from accountResult: Result<AccountData, Error>) throws -> AccountVerdict {
        do {
            let accountData = try accountResult.get()

            if accountData.expiry > Date() {
                return .good(accountData)
            } else {
                return .expired(accountData)
            }
        } catch {
            if let error = error as? REST.Error, error.compareErrorCode(.invalidAccount) {
                return .invalidAccount
            } else {
                throw error
            }
        }
    }

    private func deviceVerdict(from deviceResult: Result<Device, Error>) throws -> DeviceVerdict {
        do {
            let deviceState = try SettingsManager.readDeviceState()
            guard let deviceData = deviceState.deviceData else { throw InvalidDeviceStateError() }

            let device = try deviceResult.get()

            return deviceData.wgKeyData.privateKey.publicKey.base64Key == device.pubkey ? .good : .keyMismatch
        } catch {
            if let error = error as? REST.Error, error.compareErrorCode(.deviceNotFound) {
                return .revoked
            } else {
                throw error
            }
        }
    }
}

private struct InvalidDeviceStateError: LocalizedError {
    var errorDescription: String? {
        return "Cannot complete device check because device is no longer logged in."
    }
}
