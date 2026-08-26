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

enum SetAccountTaskAction {
    /// Set new account.
    case new

    /// Set existing account.
    case existing(String)

    /// Unset account.
    case unset(isRemovingProfile: Bool)

    /// Delete account.
    case delete(String)

    var taskName: String {
        switch self {
        case .new:
            "Set new account"
        case .existing:
            "Set existing account"
        case .unset:
            "Unset account"
        case .delete:
            "Delete account"
        }
    }
}

enum AccountStateUpdate: Sendable {
    case setLastUsedAccount(String)
    case setDeviceState(DeviceState)
    case logout(isRemovingProfile: Bool)
}

final class SetAccountTask: Sendable {
    private let accountsProxy: RESTAccountHandling
    private let devicesProxy: DeviceHandling
    private let action: SetAccountTaskAction
    private let deviceState: DeviceState

    private let updateState: @Sendable (AccountStateUpdate) async -> Void

    private let logger = Logger(label: "SetAccountOperation")

    init(
        accountsProxy: RESTAccountHandling,
        devicesProxy: DeviceHandling,
        action: SetAccountTaskAction,
        deviceState: DeviceState,
        updateState: @escaping @Sendable (AccountStateUpdate) async -> Void
    ) {
        self.accountsProxy = accountsProxy
        self.devicesProxy = devicesProxy
        self.action = action
        self.deviceState = deviceState
        self.updateState = updateState
    }

    func run() async -> Result<StoredAccountData?, Error> {
        switch action {
        case .new:
            await startLogoutFlow()

            return await startNewAccountFlow()
                .map { $0 }

        case let .existing(accountNumber):
            await startLogoutFlow()

            return await startExistingAccountFlow(
                accountNumber: accountNumber
            )
            .map { $0 }

        case let .unset(isRemovingProfile):
            await startLogoutFlow(
                isRemovingProfile: isRemovingProfile
            )

            return .success(nil)

        case let .delete(accountNumber):
            return await startDeleteAccountFlow(
                accountNumber: accountNumber
            )
        }
    }

    // MARK: - Private

    /**
     Begin logout flow by performing the following steps:

     1. Delete currently logged in device from the API if device is logged in.
     2. Transition device state to logged out state.
     3. Remove system VPN configuration if exists.
     4. Reset tunnel status to disconnected state.

     Does nothing if device is already logged out.

     The current device state is passed into the operation instead of accessing
     TunnelInteractor directly. The actor owns the actual state transition and
     persistence through the logout callback.
     */
    private func startLogoutFlow(
        isRemovingProfile: Bool = true
    ) async {
        switch deviceState {
        case let .loggedIn(accountData, deviceData):
            // Delete the currently logged-in device from the API before
            // transitioning the local device state to logged out.
            await deleteDevice(
                accountNumber: accountData.number,
                deviceIdentifier: deviceData.identifier
            )

            // Continue the logout flow after the API request completes.
            // Errors from deleting the device are intentionally ignored,
            // preserving the existing logout behavior.
            await updateState(
                .logout(isRemovingProfile: isRemovingProfile)
            )

        case .revoked:
            // A revoked device does not need to be deleted from the API.
            await updateState(
                .logout(isRemovingProfile: isRemovingProfile)
            )

        case .loggedOut:
            // Nothing needs to be done when the device is already logged out.
            break
        }
    }

    /**
     Begin login flow with a new account and performing the following steps:

     1. Create new account via API.
     2. Call `continueLoginFlow()` passing the result of account creation request.
     */
    private func startNewAccountFlow() async -> Result<StoredAccountData, Error> {
        do {
            let accountData = try await createAccount().get()

            await updateState(.setLastUsedAccount(accountData.number))

            let newDevice = try await createDevice(
                accountNumber: accountData.number
            )

            await storeSettings(
                accountData: accountData,
                newDevice: newDevice
            )

            return .success(accountData)
        } catch {
            return .failure(error)
        }
    }

    /**
     Begin login flow with an existing account by performing the following steps:

     1. Retrieve existing account from the API.
     2. Call `continueLoginFlow()` passing the result of account retrieval request.
     */
    private func startExistingAccountFlow(
        accountNumber: String
    ) async -> Result<StoredAccountData, Error> {
        do {
            let accountData = try await getAccount(
                accountNumber: accountNumber
            ).get()

            await updateState(.setLastUsedAccount(accountData.number))

            let newDevice = try await createDevice(
                accountNumber: accountData.number
            )

            await storeSettings(
                accountData: accountData,
                newDevice: newDevice
            )

            return .success(accountData)
        } catch {
            return .failure(error)
        }
    }

    /**
     Begin delete flow of an existing account by performing the following steps:

     1. Delete existing account with the API.
     2. On success, remove last used account and unset device state (logout)),
        otherwise, propagate the error.
     */
    private func startDeleteAccountFlow(
        accountNumber: String
    ) async -> Result<StoredAccountData?, Error> {
        let result = await deleteAccount(accountNumber: accountNumber)

        guard result.isSuccess else {
            return result.map { _ in nil }
        }
        await updateState(.logout(isRemovingProfile: true))

        return .success(nil)
    }

    /// Store account data and newly created device in settings and transition device state to logged in state.
    private func storeSettings(
        accountData: StoredAccountData,
        newDevice: NewDevice
    ) async {
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

        await updateState(.setDeviceState(.loggedIn(accountData, storedDeviceData)))
    }

    /// Create new account and produce `StoredAccountData` upon success.
    private func createAccount() async -> Result<StoredAccountData, Error> {
        logger.debug("Create new account...")

        let result = await accountsProxy.createAccount(
            retryStrategy: .default
        )

        return
            result
            .inspectError { [logger] error in
                guard !error.isOperationCancellationError else {
                    return
                }

                logger.error(
                    error: error,
                    message: "Failed to create new account."
                )
            }
            .map { newAccountData -> StoredAccountData in
                logger.debug("Created new account.")

                return StoredAccountData(
                    identifier: newAccountData.id,
                    number: newAccountData.number,
                    expiry: newAccountData.expiry
                )
            }
    }

    /// Get account data from the API and produce `StoredAccountData` upon success.
    private func getAccount(
        accountNumber: String
    ) async -> Result<StoredAccountData, Error> {
        logger.debug("Request account data...")

        let result = await accountsProxy.getAccountData(
            accountNumber: accountNumber,
            retryStrategy: .default
        )

        return
            result
            .inspectError { [logger] error in
                guard !(error is CancellationError) else {
                    return
                }

                logger.error(
                    error: error,
                    message: "Failed to receive account data."
                )
            }
            .map { accountData -> StoredAccountData in
                logger.debug("Received account data.")

                return StoredAccountData(
                    identifier: accountData.id,
                    number: accountNumber,
                    expiry: accountData.expiry
                )
            }
    }

    /// Delete account.
    private func deleteAccount(
        accountNumber: String
    ) async -> Result<Void, Error> {
        logger.debug("Delete account...")

        let result = await accountsProxy.deleteAccount(
            accountNumber: accountNumber,
            retryStrategy: .default
        )

        return result.inspectError { [logger] error in
            guard !error.isOperationCancellationError else {
                return
            }

            logger.error(
                error: error,
                message: "Failed to delete account."
            )
        }
    }

    /// Delete device from API.
    private func deleteDevice(
        accountNumber: String,
        deviceIdentifier: String
    ) async {
        logger.debug("Delete current device...")

        let result = await devicesProxy.deleteDevice(
            accountNumber: accountNumber,
            identifier: deviceIdentifier,
            retryStrategy: .default
        )

        switch result {
        case let .success(isDeleted):
            logger.debug(
                isDeleted
                    ? "Deleted device."
                    : "Device is already deleted."
            )

        case let .failure(error):
            if !error.isOperationCancellationError {
                logger.error(
                    error: error,
                    message: "Failed to delete device."
                )
            }
        }
    }

    /// Create new private key and create new device via API.
    private func createDevice(
        accountNumber: String
    ) async throws -> NewDevice {
        let privateKey = WireGuard.PrivateKey()
        let request = CreateDeviceRequest(
            publicKey: privateKey.publicKey,
            hijackDNS: false
        )

        logger.debug("Create device...")

        let result = await devicesProxy.createDevice(
            accountNumber: accountNumber,
            request: request,
            retryStrategy: .default
        )

        // Due to retry strategy, it's possible for server to register the new key without being
        // able to return the acknowledgment back to client.
        // In that case the subsequent retry attempt will error with `.publicKeyInUse`. Fetch the device
        // from API when that happens.
        if let error = result.error as? REST.Error,
            error.compareErrorCode(.publicKeyInUse)
        {
            guard
                let device = try await findDevice(
                    accountNumber: accountNumber,
                    publicKey: privateKey.publicKey
                )
            else {
                throw error
            }

            return NewDevice(
                privateKey: privateKey,
                device: device
            )
        }

        return
            try result
            .map {
                NewDevice(
                    privateKey: privateKey,
                    device: $0
                )
            }
            .get()
    }

    /// Find device by public key in the list of devices registered on server. The result passed to `completion` handler
    /// may contain `nil` if such device is not found for some reason.
    private func findDevice(
        accountNumber: String,
        publicKey: WireGuard.PublicKey
    ) async throws -> Device? {
        let result = await devicesProxy.getDevices(
            accountNumber: accountNumber,
            retryStrategy: .default
        )

        return
            try result
            .inspectError { [logger] error in
                logger.error(
                    error: error,
                    message: "Failed to get devices."
                )
            }
            .get()
            .first { device in
                device.pubkey == publicKey
            }
    }

    /// Struct that holds a private key that was used for creating a device via API along with the successful
    /// response from the API.
    private struct NewDevice {
        var privateKey: WireGuard.PrivateKey
        var device: Device
    }
}
