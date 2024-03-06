//
//  DeviceCheckOperation.swift
//  PacketTunnel
//
//  Created by pronebird on 20/04/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadSettings
import MullvadTypes
@testable import MullvadVPN
import Operations
import PacketTunnelCore
import WireGuardKitTypes

/**
 An operation that is responsible for performing account and device diagnostics and key rotation from within packet
 tunnel process.

 Packet tunnel runs this operation immediately as it starts, with `rotateImmediatelyOnKeyMismatch` flag set to
 `true` which forces key rotation to happpen immediately given that the key stored on server does not match the key
 stored on device. Unless the last rotation attempt took place less than 15 seconds ago in which case the key rotation
 is not performed.

 Other times, packet tunnel runs this operation with `rotateImmediatelyOnKeyMismatch` set to `false`, in which
 case it respects the 24 hour interval between key rotation retry attempts.
 */
final class DeviceCheckOperation: ResultOperation<DeviceCheck> {
    private let logger = Logger(label: "DeviceCheckOperation")

    private let remoteService: DeviceCheckRemoteServiceProtocol
    private let deviceStateAccessor: DeviceStateAccessorProtocol
    private let rotateImmediatelyOnKeyMismatch: Bool

    private var tasks: [Cancellable] = []

    init(
        dispatchQueue: DispatchQueue,
        remoteSevice: DeviceCheckRemoteServiceProtocol,
        deviceStateAccessor: DeviceStateAccessorProtocol,
        rotateImmediatelyOnKeyMismatch: Bool,
        completionHandler: CompletionHandler? = nil
    ) {
        self.remoteService = remoteSevice
        self.deviceStateAccessor = deviceStateAccessor
        self.rotateImmediatelyOnKeyMismatch = rotateImmediatelyOnKeyMismatch

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

    /**
     Begins the flow by fetching device state and then fetching account and device data. Calls `didReceiveData()` with
     the received data when done.
     */
    private func startFlow(completion: @escaping (Result<DeviceCheck, Error>) -> Void) {
        do {
            guard case let .loggedIn(accountData, deviceData) = try deviceStateAccessor.read() else {
                throw DeviceCheckError.invalidDeviceState
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

    /**
     Handles received data results and initiates key rotation when the key stored on server does not match the key
     stored on device.
     */
    private func didReceiveData(
        accountResult: Result<Account, Error>,
        deviceResult: Result<Device, Error>,
        completion: @escaping (Result<DeviceCheck, Error>) -> Void
    ) {
        do {
            let accountVerdict = try accountVerdict(from: accountResult)
            let deviceVerdict = try deviceVerdict(from: deviceResult)

            // Do not rotate the key if account is invalid even if the API successfully returns a device.
            if accountVerdict != .invalid, deviceVerdict == .keyMismatch {
                rotateKeyIfNeeded { rotationResult in
                    completion(rotationResult.map { rotationStatus in
                        DeviceCheck(
                            accountVerdict: accountVerdict,
                            deviceVerdict: rotationStatus.isSucceeded ? .active : .keyMismatch,
                            keyRotationStatus: rotationStatus
                        )
                    })
                }
            } else {
                completion(.success(DeviceCheck(
                    accountVerdict: accountVerdict,
                    deviceVerdict: deviceVerdict,
                    keyRotationStatus: .noAction
                )))
            }
        } catch {
            completion(.failure(error))
        }
    }

    // MARK: - Data fetch

    /// Fetch account and device data simultaneously, upon completion calls completion handler passing the results to
    /// it.
    private func fetchData(
        accountNumber: String, deviceIdentifier: String,
        completion: @escaping (Result<Account, Error>, Result<Device, Error>) -> Void
    ) {
        var accountResult: Result<Account, Error> = .failure(OperationError.cancelled)
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

    /**
     Checks if the key should be rotated by checking when the last rotation took place. If conditions are satisfied,
     then it rotate device key by marking the beginning of key rotation, updating device state and persisting before
     proceeding to rotate the key.
     */
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
            completion(.failure(DeviceCheckError.invalidDeviceState))
            return
        }

        var keyRotation = WgKeyRotation(data: deviceData)
        guard keyRotation.shouldRotateFromPacketTunnel(rotateImmediately: rotateImmediatelyOnKeyMismatch) else {
            completion(.success(.noAction))
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

    /**
     Updates device state with the new data received from `Device` and marks key rotation as completed by swapping the
     current private key and erasing information about the last key rotation attempt.
     */
    private func completeKeyRotation(_ device: Device) throws {
        logger.debug("Successfully rotated device key. Persisting device state...")

        let deviceState = try deviceStateAccessor.read()
        guard case let .loggedIn(accountData, deviceData) = deviceState else {
            logger.debug("Will not persist device state after rotating the key because device is no longer logged in.")
            throw DeviceCheckError.invalidDeviceState
        }

        var keyRotation = WgKeyRotation(data: deviceData)
        let isCompleted = keyRotation.setCompleted(with: device)

        if isCompleted {
            do {
                try deviceStateAccessor.write(.loggedIn(accountData, keyRotation.data))
            } catch {
                logger.error(error: error, message: "Failed to persist device state after rotating the key.")
                throw error
            }
        } else {
            logger.debug("Cannot complete key rotation due to rotation race.")

            throw DeviceCheckError.keyRotationRace
        }
    }

    // MARK: - Private helpers

    /// Converts account data result type into `AccountVerdict`.
    private func accountVerdict(from accountResult: Result<Account, Error>) throws -> AccountVerdict {
        do {
            let account = try accountResult.get()

            return account.expiry > Date() ? .active(account) : .expired(account)
        } catch let error as REST.Error where error.compareErrorCode(.invalidAccount) {
            return .invalid
        }
    }

    /// Converts device result type into `DeviceVerdict`.
    private func deviceVerdict(from deviceResult: Result<Device, Error>) throws -> DeviceVerdict {
        do {
            let deviceState = try deviceStateAccessor.read()
            guard let deviceData = deviceState.deviceData else { throw DeviceCheckError.invalidDeviceState }

            let device = try deviceResult.get()

            return deviceData.wgKeyData.privateKey.publicKey == device.pubkey ? .active : .keyMismatch
        } catch let error as REST.Error where error.compareErrorCode(.deviceNotFound) {
            return .revoked
        }
    }
}

/// An error used internally by `DeviceCheckOperation`.
private enum DeviceCheckError: LocalizedError, Equatable {
    /// Device is no longer logged in.
    case invalidDeviceState

    /// Main process has likely performed key rotation at the same time when packet tunnel was doing so.
    case keyRotationRace

    var errorDescription: String? {
        switch self {
        case .invalidDeviceState:
            return "Cannot complete device check because device is no longer logged in."
        case .keyRotationRace:
            return "Detected key rotation race condition."
        }
    }
}
