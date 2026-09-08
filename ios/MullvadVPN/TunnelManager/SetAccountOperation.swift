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

enum SetAccountAction {
    /// Set new account.
    case new

    /// Set existing account.
    case existing(String)

    /// Unset account.
    case unset

    /// Delete account.
    case delete(String)

    var taskName: String {
        switch self {
        case .new: "Set new account"
        case .existing: "Set existing account"
        case .unset: "Unset account"
        case .delete: "Delete account"
        }
    }
}

class SetAccountOperation: ResultOperation<StoredAccountData?>, @unchecked Sendable {
    private let accountsProxy: RESTAccountHandling
    private let devicesProxy: DeviceHandling
    private let action: SetAccountAction
    private let deviceState: @Sendable () -> DeviceState
    private let onUpdateAccount: @Sendable (DeviceState) -> Void

    private let logger = Logger(label: "SetAccountOperation")
    private var tasks: [Cancellable] = []

    init(
        dispatchQueue: DispatchQueue,
        accountsProxy: RESTAccountHandling,
        devicesProxy: DeviceHandling,
        action: SetAccountAction,
        deviceState: @escaping @Sendable () -> DeviceState,
        onUpdateAccount: @escaping @Sendable (DeviceState) -> Void
    ) {
        self.accountsProxy = accountsProxy
        self.devicesProxy = devicesProxy
        self.action = action
        self.onUpdateAccount = onUpdateAccount
        self.deviceState = deviceState

        super.init(dispatchQueue: dispatchQueue)
    }

    // MARK: -

    override func main() {
        switch action {
        case .new:
            startLogoutFlow { [self] in
                startNewAccountFlow { [self] result in
                    finish(result: result.map { .some($0) })
                }
            }

        case let .existing(accountNumber):
            startLogoutFlow { [self] in
                startExistingAccountFlow(accountNumber: accountNumber) { [self] result in
                    finish(result: result.map { .some($0) })
                }
            }

        case .unset:
            startLogoutFlow { [self] in
                finish(result: .success(nil))
            }

        case let .delete(accountNumber):
            startDeleteAccountFlow(accountNumber: accountNumber) { [self] result in
                finish(result: result.map { .none })
            }
        }
    }

    override func operationDidCancel() {
        tasks.forEach { $0.cancel() }
        tasks.removeAll()
    }

    // MARK: - Private

    /**
     Begin logout flow by performing the following steps:

     1. Delete currently logged in device from the API if device is logged in.
     2. Transition device state to logged out state.
     3. Remove system VPN configuration if exists.
     4. Reset tunnel status to disconnected state.

     Does nothing if device is already logged out.
     */
    private func startLogoutFlow(isRemovingProfile: Bool = true, completion: @escaping @Sendable () -> Void) {
        switch deviceState() {
        case let .loggedIn(accountData, deviceData):
            deleteDevice(accountNumber: accountData.number, deviceIdentifier: deviceData.identifier) { [self] _ in
                onUpdateAccount(.loggedOut)
                completion()
            }

        case .revoked:
            onUpdateAccount(.loggedOut)
            completion()

        case .loggedOut:
            completion()
        }
    }

    /**
     Begin login flow with a new account and performing the following steps:

     1. Create new account via API.
     2. Call `continueLoginFlow()` passing the result of account creation request.
     */
    private func startNewAccountFlow(completion: @escaping @Sendable (Result<StoredAccountData, Error>) -> Void) {
        createAccount { [self] result in
            continueLoginFlow(result, completion: completion)
        }
    }

    /**
     Begin login flow with an existing account by performing the following steps:

     1. Retrieve existing account from the API.
     2. Call `continueLoginFlow()` passing the result of account retrieval request.
     */
    private func startExistingAccountFlow(
        accountNumber: String,
        completion: @escaping @Sendable (Result<StoredAccountData, Error>) -> Void
    ) {
        getAccount(accountNumber: accountNumber) { [self] result in
            continueLoginFlow(result, completion: completion)
        }
    }

    /**
     Begin delete flow of an existing account by performing the following steps:

     1. Delete existing account with the API.
     2. On success, remove last used account and unset device state (logout)),
       otherwise, propagate the error.
     */
    private func startDeleteAccountFlow(
        accountNumber: String,
        completion: @escaping @Sendable (Result<Void, Error>) -> Void
    ) {
        deleteAccount(accountNumber: accountNumber) { [self] result in
            if result.isSuccess {
                onUpdateAccount(.loggedOut)
                completion(result)
            } else {
                completion(result)
            }
        }
    }

    /**
     Continue login flow after receiving account data as a part of creating new or retrieving existing account from
     the API by performing the following steps:

     1. Store last used account number.
     2. Create new device with the API.
     3. Persist settings.
     */
    private func continueLoginFlow(
        _ result: Result<StoredAccountData, Error>,
        completion: @escaping @Sendable (Result<StoredAccountData, Error>) -> Void
    ) {
        do {
            let accountData = try result.get()
            createDevice(accountNumber: accountData.number) { [self] result in
                completion(
                    result.map { newDevice in
                        storeSettings(accountData: accountData, newDevice: newDevice)

                        return accountData
                    })
            }
        } catch {
            completion(.failure(error))
        }
    }

    /// Store account data and newly created device in settings and transition device state to logged in state.
    private func storeSettings(accountData: StoredAccountData, newDevice: NewDevice) {
        logger.debug("Saving settings...")

        // Create stored device data.
        let restDevice = newDevice.device
        let storedDeviceData = StoredDeviceData(
            creationDate: restDevice.created,
            identifier: restDevice.id,
            name: restDevice.name,
            hijackDNS: restDevice.hijackDNS,
            ipv4Address: restDevice.ipv4Address,
            ipv6Address: restDevice.ipv6Address,
            wgKeyData: StoredWgKeyData(
                creationDate: Date(),
                privateKey: newDevice.privateKey
            )
        )

        // Transition device state to logged in.
        onUpdateAccount(.loggedIn(accountData, storedDeviceData))
    }

    /// Create new account and produce `StoredAccountData` upon success.
    private func createAccount(completion: @escaping @Sendable (Result<StoredAccountData, Error>) -> Void) {
        logger.debug("Create new account...")

        let task = accountsProxy.createAccount(retryStrategy: .default) { [self] result in
            dispatchQueue.async { [self] in
                let result = result.inspectError { error in
                    guard !error.isOperationCancellationError else { return }

                    logger.error(error: error, message: "Failed to create new account.")
                }.map { newAccountData -> StoredAccountData in
                    logger.debug("Created new account.")

                    return StoredAccountData(
                        identifier: newAccountData.id,
                        number: newAccountData.number,
                        expiry: newAccountData.expiry
                    )
                }

                completion(result)
            }
        }

        tasks.append(task)
    }

    /// Get account data from the API and produce `StoredAccountData` upon success.
    private func getAccount(
        accountNumber: String,
        completion: @escaping @Sendable (Result<StoredAccountData, Error>) -> Void
    ) {
        logger.debug("Request account data...")

        let task = Task {
            let accountResult = await accountsProxy.getAccountData(
                accountNumber: accountNumber,
                retryStrategy: .default
            )

            let result = accountResult.inspectError { error in
                guard !(error is CancellationError) else { return }

                logger.error(error: error, message: "Failed to receive account data.")
            }.map { accountData -> StoredAccountData in
                logger.debug("Received account data.")

                return StoredAccountData(
                    identifier: accountData.id,
                    number: accountNumber,
                    expiry: accountData.expiry
                )
            }

            dispatchQueue.async {
                completion(result)
            }
        }

        tasks.append(task.cancellable)
    }

    /// Delete account.
    private func deleteAccount(accountNumber: String, completion: @escaping @Sendable (Result<Void, Error>) -> Void) {
        logger.debug("Delete account...")

        let task = accountsProxy.deleteAccount(
            accountNumber: accountNumber,
            retryStrategy: .default
        ) { [self] result in
            dispatchQueue.async { [self] in
                let result = result.inspectError { error in
                    guard !error.isOperationCancellationError else { return }

                    logger.error(error: error, message: "Failed to delete account.")
                }

                completion(result)
            }
        }

        tasks.append(task)
    }

    /// Delete device from API.
    private func deleteDevice(
        accountNumber: String,
        deviceIdentifier: String,
        completion: @escaping @Sendable (Error?) -> Void
    ) {
        logger.debug("Delete current device...")

        let task = devicesProxy.deleteDevice(
            accountNumber: accountNumber,
            identifier: deviceIdentifier,
            retryStrategy: .default
        ) { [self] result in
            dispatchQueue.async { [self] in
                switch result {
                case let .success(isDeleted):
                    logger.debug(isDeleted ? "Deleted device." : "Device is already deleted.")

                case let .failure(error):
                    if !error.isOperationCancellationError {
                        logger.error(error: error, message: "Failed to delete device.")
                    }
                }

                completion(result.error)
            }
        }

        tasks.append(task)
    }

    /// Create new private key and create new device via API.
    private func createDevice(
        accountNumber: String,
        completion: @escaping @Sendable (Result<NewDevice, Error>) -> Void
    ) {
        let privateKey = WireGuard.PrivateKey()
        let request = CreateDeviceRequest(publicKey: privateKey.publicKey, hijackDNS: false)

        logger.debug("Create device...")

        let task =
            devicesProxy
            .createDevice(accountNumber: accountNumber, request: request, retryStrategy: .default) { [self] result in
                dispatchQueue.async { [self] in
                    // Due to retry strategy, it's possible for server to register the new key without being
                    // able to return the acknowledgment back to client.
                    // In that case the subsequent retry attempt will error with `.publicKeyInUse`. Fetch the device
                    // from API when that happens.
                    if let error = result.error as? REST.Error, error.compareErrorCode(.publicKeyInUse) {
                        self.findDevice(accountNumber: accountNumber, publicKey: privateKey.publicKey) { result in
                            let result = result.flatMap { device in
                                if let device {
                                    return .success(NewDevice(privateKey: privateKey, device: device))
                                } else {
                                    return .failure(error)
                                }
                            }
                            completion(result)
                        }
                    } else {
                        completion(result.map { NewDevice(privateKey: privateKey, device: $0) })
                    }
                }
            }

        tasks.append(task)
    }

    /// Find device by public key in the list of devices registered on server. The result passed to `completion` handler
    /// may contain `nil` if such device is not found for some reason.
    private func findDevice(
        accountNumber: String,
        publicKey: WireGuard.PublicKey,
        completion: @escaping @Sendable (Result<Device?, Error>) -> Void
    ) {
        let task = devicesProxy.getDevices(accountNumber: accountNumber, retryStrategy: .default) { [self] result in
            dispatchQueue.async { [self] in
                let result =
                    result
                    .flatMap { devices in
                        .success(
                            devices.first { device in
                                device.pubkey == publicKey
                            })
                    }
                    .inspectError { error in
                        logger.error(error: error, message: "Failed to get devices.")
                    }

                completion(result)
            }
        }

        tasks.append(task)
    }

    /// Struct that holds a private key that was used for creating a new device on the API along with the successful
    /// response from the API.
    private struct NewDevice {
        var privateKey: WireGuard.PrivateKey
        var device: Device
    }
}
