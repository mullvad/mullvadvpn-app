//
//  SetAccountOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 16/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadSettings
import MullvadTypes
import Operations
import WireGuardKitTypes

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

    // true if this action results in an account being set if successful
    var isConstructive: Bool {
        switch self {
        case .unset, .delete: false
        default: true
        }
    }
}

class SetAccountOperation: ResultOperation<StoredAccountData?> {
    private let interactor: TunnelInteractor
    private let accountsProxy: RESTAccountHandling
    private let devicesProxy: DeviceHandling
    private let action: SetAccountAction
    private let accessTokenManager: RESTAccessTokenManagement

    private let logger = Logger(label: "SetAccountOperation")
    private var tasks: [Cancellable] = []

    init(
        dispatchQueue: DispatchQueue,
        interactor: TunnelInteractor,
        accountsProxy: RESTAccountHandling,
        devicesProxy: DeviceHandling,
        accessTokenManager: RESTAccessTokenManagement,
        action: SetAccountAction
    ) {
        self.interactor = interactor
        self.accountsProxy = accountsProxy
        self.devicesProxy = devicesProxy
        self.accessTokenManager = accessTokenManager
        self.action = action

        super.init(dispatchQueue: dispatchQueue)
    }

    // MARK: -

    override func main() {
        startLogoutFlow { [self] in
            self.accessTokenManager.invalidateAllTokens()
            switch action {
            case .new:
                startNewAccountFlow { [self] result in
                    finish(result: result.map { .some($0) })
                }

            case let .existing(accountNumber):
                startExistingAccountFlow(accountNumber: accountNumber) { [self] result in
                    finish(result: result.map { .some($0) })
                }

            case .unset:
                finish(result: .success(nil))

            case let .delete(accountNumber):
                startDeleteAccountFlow(accountNumber: accountNumber) { [self] result in
                    finish(result: result.map { .none })
                }
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
    private func startLogoutFlow(completion: @escaping () -> Void) {
        switch interactor.deviceState {
        case let .loggedIn(accountData, deviceData):
            deleteDevice(accountNumber: accountData.number, deviceIdentifier: deviceData.identifier) { [self] _ in
                unsetDeviceState(completion: completion)
            }

        case .revoked:
            unsetDeviceState(completion: completion)

        case .loggedOut:
            completion()
        }
    }

    /**
     Begin login flow with a new account and performing the following steps:

     1. Create new account via API.
     2. Call `continueLoginFlow()` passing the result of account creation request.
     */
    private func startNewAccountFlow(completion: @escaping (Result<StoredAccountData, Error>) -> Void) {
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
        completion: @escaping (Result<StoredAccountData, Error>) -> Void
    ) {
        getAccount(accountNumber: accountNumber) { [self] result in
            continueLoginFlow(result, completion: completion)
        }
    }

    /**
     Begin delete flow of an existing account by performing the following steps:

     1. Delete existing account with the API.
     2. Reset tunnel settings to default and remove last used account.
     */
    private func startDeleteAccountFlow(
        accountNumber: String,
        completion: @escaping (Result<Void, Error>) -> Void
    ) {
        deleteAccount(accountNumber: accountNumber) { [self] result in
            interactor.setSettings(LatestTunnelSettings(), persist: true)

            if result.isSuccess {
                interactor.removeLastUsedAccount()
            }

            completion(result)
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
        completion: @escaping (Result<StoredAccountData, Error>) -> Void
    ) {
        do {
            let accountData = try result.get()

            storeLastUsedAccount(accountNumber: accountData.number)

            createDevice(accountNumber: accountData.number) { [self] result in
                completion(result.map { newDevice in
                    storeSettings(accountData: accountData, newDevice: newDevice)

                    return accountData
                })
            }
        } catch {
            completion(.failure(error))
        }
    }

    /// Store last used account number in settings.
    /// Errors are ignored but logged.
    private func storeLastUsedAccount(accountNumber: String) {
        logger.debug("Store last used account.")

        do {
            try SettingsManager.setLastUsedAccount(accountNumber)
        } catch {
            logger.error(error: error, message: "Failed to store last used account number.")
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

        // Reset tunnel settings.
        interactor.setSettings(LatestTunnelSettings(), persist: true)

        // Transition device state to logged in.
        interactor.setDeviceState(.loggedIn(accountData, storedDeviceData), persist: true)
    }

    /// Create new account and produce `StoredAccountData` upon success.
    private func createAccount(completion: @escaping (Result<StoredAccountData, Error>) -> Void) {
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
    private func getAccount(accountNumber: String, completion: @escaping (Result<StoredAccountData, Error>) -> Void) {
        logger.debug("Request account data...")

        let task = accountsProxy.getAccountData(accountNumber: accountNumber).execute(
            retryStrategy: .default
        ) { [self] result in
            dispatchQueue.async { [self] in
                let result = result.inspectError { error in
                    guard !error.isOperationCancellationError else { return }

                    logger.error(error: error, message: "Failed to receive account data.")
                }.map { accountData -> StoredAccountData in
                    logger.debug("Received account data.")

                    return StoredAccountData(
                        identifier: accountData.id,
                        number: accountNumber,
                        expiry: accountData.expiry
                    )
                }

                completion(result)
            }
        }

        tasks.append(task)
    }

    /// Delete account.
    private func deleteAccount(accountNumber: String, completion: @escaping (Result<Void, Error>) -> Void) {
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
    private func deleteDevice(accountNumber: String, deviceIdentifier: String, completion: @escaping (Error?) -> Void) {
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

    /**
     Transitions device state into logged out state by performing the following tasks:

     1. Prepare tunnel manager for removal of VPN configuration. In response tunnel manager stops processing VPN status
        notifications coming from VPN configuration.
     2. Reset device staate to logged out and persist it.
     3. Remove VPN configuration and release an instance of `Tunnel` object.
     */
    private func unsetDeviceState(completion: @escaping () -> Void) {
        // Tell the caller to unsubscribe from VPN status notifications.
        interactor.prepareForVPNConfigurationDeletion()

        // Reset tunnel and device state.
        interactor.updateTunnelStatus { tunnelStatus in
            tunnelStatus = TunnelStatus()
            tunnelStatus.state = .disconnected
        }
        interactor.setDeviceState(.loggedOut, persist: true)

        // Finish immediately if tunnel provider is not set.
        guard let tunnel = interactor.tunnel else {
            completion()
            return
        }

        // Remove VPN configuration.
        tunnel.removeFromPreferences { [self] error in
            dispatchQueue.async { [self] in
                // Ignore error but log it.
                if let error {
                    logger.error(error: error, message: "Failed to remove VPN configuration.")
                }

                interactor.setTunnel(nil, shouldRefreshTunnelState: false)

                completion()
            }
        }
    }

    /// Create new private key and create new device via API.
    private func createDevice(accountNumber: String, completion: @escaping (Result<NewDevice, Error>) -> Void) {
        let privateKey = PrivateKey()
        let request = REST.CreateDeviceRequest(publicKey: privateKey.publicKey, hijackDNS: false)

        logger.debug("Create device...")

        let task = devicesProxy
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
        publicKey: PublicKey,
        completion: @escaping (Result<Device?, Error>) -> Void
    ) {
        let task = devicesProxy.getDevices(accountNumber: accountNumber, retryStrategy: .default) { [self] result in
            dispatchQueue.async { [self] in
                let result = result
                    .flatMap { devices in
                        .success(devices.first { device in
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
        var privateKey: PrivateKey
        var device: Device
    }
}

// swiftlint:disable:this file_length
