//
//  MigrateSettingsOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 18/05/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import Operations
import class WireGuardKitTypes.PrivateKey

class MigrateSettingsOperation: AsyncOperation {
    private let accountTokenKey = "accountToken"
    private let accountExpiryKey = "accountExpiry"

    private let accountsProxy: REST.AccountsProxy
    private let devicesProxy: REST.DevicesProxy

    private let logger = Logger(label: "MigrateSettingsOperation")

    private var accountTask: Cancellable?
    private var deviceTask: Cancellable?

    private var accountData: REST.AccountData?
    private var devices: [REST.Device]?

    init(
        dispatchQueue: DispatchQueue,
        accountsProxy: REST.AccountsProxy,
        devicesProxy: REST.DevicesProxy
    ) {
        self.accountsProxy = accountsProxy
        self.devicesProxy = devicesProxy

        super.init(dispatchQueue: dispatchQueue)
    }

    override func main() {
        // Read legacy account number from user defaults.
        let storedAccountNumber = UserDefaults.standard.string(forKey: accountTokenKey)

        guard let storedAccountNumber = storedAccountNumber else {
            logger.debug("Account number is not found in user defaults. Nothing to migrate.")

            finishMigration()
            return
        }

        // Set legacy account number as last used.
        logger.debug("Found legacy account number.")
        logger.debug("Store last used account.")

        do {
            try SettingsManager.setLastUsedAccount(storedAccountNumber)
        } catch {
            logger.error(
                error: error,
                message: "Failed to store last used account."
            )
        }

        // List legacy settings stored in keychain.
        logger.debug("Read legacy settings...")

        var storedSettings: [LegacyTunnelSettings] = []
        do {
            storedSettings = try SettingsManager.readLegacySettings()
        } catch .itemNotFound as KeychainError {
            logger.debug("Legacy settings are not found in keychain.")

            finishMigration()
            return
        } catch {
            logger.error(
                error: error,
                message: "Failed to read legacy settings from keychain."
            )
            finishMigration()
            return
        }

        // Find settings matching the account number stored in user defaults.
        let matchingSettings = storedSettings.first { settings in
            return settings.accountNumber == storedAccountNumber
        }

        guard let matchingSettings = matchingSettings else {
            logger.debug(
                "Could not find legacy settings matching the legacy account number."
            )

            finishMigration()
            return
        }

        // Fetch remote data concurrently.
        logger.debug("Fetching account and device data...")

        let dispatchGroup = DispatchGroup()

        dispatchGroup.enter()
        accountTask = accountsProxy.getAccountData(
            accountNumber: storedAccountNumber,
            retryStrategy: .aggressive
        ) { completion in
            self.dispatchQueue.async {
                self.didFinishAccountRequest(completion)

                dispatchGroup.leave()
            }
        }

        dispatchGroup.enter()
        deviceTask = devicesProxy.getDevices(
            accountNumber: storedAccountNumber,
            retryStrategy: .aggressive
        ) { completion in
            self.dispatchQueue.async {
                self.didFinishDeviceRequest(completion)

                dispatchGroup.leave()
            }
        }

        dispatchGroup.notify(queue: dispatchQueue) {
            // Migrate settings if all data is available.
            if let accountData = self.accountData, let devices = self.devices {
                self.migrateSettings(
                    settings: matchingSettings,
                    accountData: accountData,
                    devices: devices
                )
            }

            // Finish migration.
            self.finishMigration()
        }
    }

    private func didFinishAccountRequest(_ completion: OperationCompletion<
        REST.AccountData,
        REST.Error
    >) {
        switch completion {
        case let .success(accountData):
            self.accountData = accountData

        case let .failure(error):
            logger.error(error: error, message: "Failed to fetch accound data.")

        case .cancelled:
            logger.debug("Account data request was cancelled.")
        }
    }

    private func didFinishDeviceRequest(_ completion: OperationCompletion<
        [REST.Device],
        REST.Error
    >) {
        switch completion {
        case let .success(devices):
            self.devices = devices

        case let .failure(error):
            logger.error(error: error, message: "Failed to fetch devices.")

        case .cancelled:
            logger.debug("Device request was cancelled.")
        }
    }

    private func migrateSettings(
        settings: LegacyTunnelSettings,
        accountData: REST.AccountData,
        devices: [REST.Device]
    ) {
        let tunnelSettings = settings.tunnelSettings
        let interfaceData = settings.tunnelSettings.interface

        // Find device that matches the public key stored in legacy settings.
        let device = devices.first { device in
            return device.pubkey == interfaceData.privateKey.publicKey ||
                device.pubkey == interfaceData.nextPrivateKey?.publicKey
        }

        guard let device = device else {
            logger.debug(
                "Failed to match legacy settings against available devices."
            )
            return
        }

        logger.debug("Found device matching public key stored in legacy settings.")

        // Match private key.
        let privateKeyWithMetadata: PrivateKeyWithMetadata
        if let nextKey = interfaceData.nextPrivateKey, nextKey.publicKey == device.pubkey {
            privateKeyWithMetadata = nextKey
        } else {
            privateKeyWithMetadata = interfaceData.privateKey
        }

        logger.debug("Store new settings...")

        // Create new settings.
        let newDeviceState = DeviceState.loggedIn(
            StoredAccountData(
                identifier: accountData.id,
                number: settings.accountNumber,
                expiry: accountData.expiry
            ),
            StoredDeviceData(
                creationDate: device.created,
                identifier: device.id,
                name: device.name,
                hijackDNS: device.hijackDNS,
                ipv4Address: device.ipv4Address,
                ipv6Address: device.ipv6Address,
                wgKeyData: StoredWgKeyData(
                    creationDate: privateKeyWithMetadata.creationDate,
                    privateKey: privateKeyWithMetadata.privateKey
                )
            )
        )

        let newSettings = TunnelSettingsV2(
            relayConstraints: tunnelSettings.relayConstraints,
            dnsSettings: interfaceData.dnsSettings
        )

        // Save settings.
        do {
            try SettingsManager.writeSettings(newSettings)
            try SettingsManager.writeDeviceState(newDeviceState)
        } catch {
            logger.error(
                error: error,
                message: "Failed to write migrated settings."
            )
        }
    }

    private func finishMigration() {
        let userDefaults = UserDefaults.standard

        logger.debug("Remove legacy settings from keychain.")
        SettingsManager.deleteLegacySettings()

        logger.debug("Remove legacy settings from user defaults.")
        userDefaults.removeObject(forKey: accountTokenKey)
        userDefaults.removeObject(forKey: accountExpiryKey)

        finish()
    }
}
