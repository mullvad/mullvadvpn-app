//
//  MigrationFromV1ToV2.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-11-18.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadTypes
import Operations

class MigrationFromV1ToV2: Migration {
    private let accountsProxy: REST.AccountsProxy
    private let devicesProxy: REST.DevicesProxy

    private var accountTask: Cancellable?
    private var deviceTask: Cancellable?

    private var accountData: REST.AccountData?
    private var devices: [REST.Device]?

    private let dispatchQueue = DispatchQueue(label: "Migration.internalQueue")

    private let legacySettings: LegacyTunnelSettings

    private let logger: Logger

    init(
        restFactory: REST.ProxyFactory,
        legacySettings: LegacyTunnelSettings,
        logger: Logger
    ) {
        accountsProxy = restFactory.createAccountsProxy()
        devicesProxy = restFactory.createDevicesProxy()
        self.logger = logger
        self.legacySettings = legacySettings
    }

    func migrate(
        with middleware: SettingsStorageMiddleware,
        completion: @escaping (Error?) -> Void
    ) {
        let storedAccountNumber = legacySettings.accountNumber

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
                    middleware: middleware,
                    settings: self.legacySettings,
                    accountData: accountData,
                    devices: devices
                )
            }

            // Finish migration.
            completion(nil)
        }
    }

    private func didFinishAccountRequest(
        _ completion: OperationCompletion<
            REST.AccountData,
            REST.Error
        >
    ) {
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
        middleware: SettingsStorageMiddleware,
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
            try middleware.saveSettings(newSettings)
            try middleware.saveDeviceState(newDeviceState)
        } catch {
            logger.error(
                error: error,
                message: "Failed to write migrated settings."
            )
        }
    }
}
