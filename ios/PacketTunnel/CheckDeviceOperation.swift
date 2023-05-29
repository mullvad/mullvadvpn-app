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

    private let remoteService: DeviceCheckRemoteServiceProtocol
    private let deviceStateAccessor: DeviceStateAccessorProtocol
    private let shouldImmediatelyRotateKeyOnMismatch: Bool

    private var tasks: [Cancellable] = []

    init(
        dispatchQueue: DispatchQueue,
        remoteSevice: DeviceCheckRemoteServiceProtocol,
        deviceStateAccessor: DeviceStateAccessorProtocol,
        shouldImmediatelyRotateKeyOnMismatch: Bool,
        completionHandler: @escaping CompletionHandler
    ) {
        self.remoteService = remoteSevice
        self.deviceStateAccessor = deviceStateAccessor
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
            guard case let .loggedIn(accountData, deviceData) = try deviceStateAccessor.read() else {
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
        let accountTask = remoteService.getAccountData(accountNumber: accountNumber) { result in
            accountResult = result
            dispatchGroup.leave()
        }

        dispatchGroup.enter()
        let deviceTask = remoteService.getDevice(accountNumber: accountNumber, identifier: deviceIdentifier) { result in
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
            deviceState = try deviceStateAccessor.read()
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
            try deviceStateAccessor.write(.loggedIn(accountData, keyRotation.data))
        } catch {
            logger.error(error: error, message: "Failed to persist updated device state before rotating the key.")
            completion(.failure(error))
            return
        }

        logger.debug("Rotate private key from packet tunnel.")

        let task = remoteService.rotateDeviceKey(
            accountNumber: accountData.number,
            identifier: deviceData.identifier,
            publicKey: publicKey
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

        let deviceState = try deviceStateAccessor.read()
        guard case let .loggedIn(accountData, deviceData) = deviceState else {
            logger.debug("Will not persist device state after rotating the key because device is no longer logged in.")
            throw InvalidDeviceStateError()
        }

        var keyRotation = WgKeyRotation(data: deviceData)
        keyRotation.setCompleted(with: device)

        do {
            try deviceStateAccessor.write(.loggedIn(accountData, keyRotation.data))
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
            let deviceState = try deviceStateAccessor.read()
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

/// A protocol that formalizes remote service dependency used by `CheckDeviceOperation`.
protocol DeviceCheckRemoteServiceProtocol {
    func getAccountData(accountNumber: String, completion: @escaping (Result<AccountData, Error>) -> Void)
        -> Cancellable
    func getDevice(accountNumber: String, identifier: String, completion: @escaping (Result<Device, Error>) -> Void)
        -> Cancellable
    func rotateDeviceKey(
        accountNumber: String,
        identifier: String,
        publicKey: PublicKey,
        completion: @escaping (Result<Device, Error>) -> Void
    ) -> Cancellable
}

/// A protocol formalizes device state accessor dependency used by `CheckDeviceOperation`.
protocol DeviceStateAccessorProtocol {
    func read() throws -> DeviceState
    func write(_ deviceState: DeviceState) throws
}

/// An object that implements remote service used by `CheckDeviceOperation`.
struct DeviceCheckRemoteService: DeviceCheckRemoteServiceProtocol {
    private let accountsProxy: REST.AccountsProxy
    private let devicesProxy: REST.DevicesProxy

    init(accountsProxy: REST.AccountsProxy, devicesProxy: REST.DevicesProxy) {
        self.accountsProxy = accountsProxy
        self.devicesProxy = devicesProxy
    }

    func getAccountData(
        accountNumber: String,
        completion: @escaping (Result<AccountData, Error>) -> Void
    ) -> Cancellable {
        accountsProxy.getAccountData(accountNumber: accountNumber, retryStrategy: .noRetry, completion: completion)
    }

    func getDevice(
        accountNumber: String,
        identifier: String,
        completion: @escaping (Result<Device, Error>) -> Void
    ) -> Cancellable {
        devicesProxy.getDevice(
            accountNumber: accountNumber,
            identifier: identifier,
            retryStrategy: .noRetry,
            completion: completion
        )
    }

    func rotateDeviceKey(
        accountNumber: String,
        identifier: String,
        publicKey: PublicKey,
        completion: @escaping (Result<Device, Error>) -> Void
    ) -> Cancellable {
        devicesProxy.rotateDeviceKey(
            accountNumber: accountNumber,
            identifier: identifier,
            publicKey: publicKey,
            retryStrategy: .default,
            completion: completion
        )
    }
}

/// An object that implements access to `DeviceState`.
struct DeviceStateAccessor: DeviceStateAccessorProtocol {
    func read() throws -> DeviceState {
        return try SettingsManager.readDeviceState()
    }

    func write(_ deviceState: DeviceState) throws {
        try SettingsManager.writeDeviceState(deviceState)
    }
}
