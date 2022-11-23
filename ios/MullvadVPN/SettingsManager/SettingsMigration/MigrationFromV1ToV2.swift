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

    private var accountCompletion: OperationCompletion<REST.AccountData, REST.Error>?
    private var devicesCompletion: OperationCompletion<[REST.Device], REST.Error>?

    private let legacySettings: LegacyTunnelSettings

    private let logger = Logger(label: "Migration.V1ToV2")

    init(
        restFactory: REST.ProxyFactory,
        legacySettings: LegacyTunnelSettings
    ) {
        accountsProxy = restFactory.createAccountsProxy()
        devicesProxy = restFactory.createDevicesProxy()
        self.legacySettings = legacySettings
    }

    func migrate(
        with store: SettingsStore,
        parser: SettingsParser,
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
            self.accountCompletion = completion

            dispatchGroup.leave()
        }

        dispatchGroup.enter()
        deviceTask = devicesProxy.getDevices(
            accountNumber: storedAccountNumber,
            retryStrategy: .aggressive
        ) { completion in
            self.devicesCompletion = completion

            dispatchGroup.leave()
        }

        dispatchGroup.notify(queue: .main) {
            switch (self.accountCompletion, self.devicesCompletion) {
            case let (.success(accountData), .success(deviceData)):
                // Migrate settings if all data is available.

                let result = Result {
                    try self.migrateSettings(
                        store: store,
                        parser: parser,
                        settings: self.legacySettings,
                        accountData: accountData,
                        devices: deviceData
                    )
                }

                completion(result.error)

            default:
                let errors = [self.accountCompletion?.error, self.devicesCompletion?.error]
                    .compactMap { $0 }
                completion(MigrateLegacySettingsError(underlyingErrors: errors))
            }
        }
    }

    private func migrateSettings(
        store: SettingsStore,
        parser: SettingsParser,
        settings: LegacyTunnelSettings,
        accountData: REST.AccountData,
        devices: [REST.Device]
    ) throws {
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
        let settingsData = try parser.producePayload(
            newSettings,
            version: SchemaVersion.v2.rawValue
        )
        let deviceData = try parser.produceUnversionedPayload(newDeviceState)

        try store.write(settingsData, for: .settings)
        try store.write(deviceData, for: .deviceState)
    }
}

struct MigrateLegacySettingsError: WrappingError {
    let underlyingErrors: [REST.Error]

    var underlyingError: Error? {
        return underlyingErrors.first
    }

    var errorDescription: String? {
        return "Failed to migrate legacy settings to v2"
    }
}
