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

final class MigrationFromV1ToV2: Migration {
    private let accountsProxy: REST.AccountsProxy
    private let devicesProxy: REST.DevicesProxy

    private var accountTask: Cancellable?
    private var deviceTask: Cancellable?

    private var accountCompletion: OperationCompletion<REST.AccountData, REST.Error> = .cancelled
    private var devicesCompletion: OperationCompletion<[REST.Device], REST.Error> = .cancelled

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

        // Store last used account number.
        logger.debug("Store legacy account number as last used account.")
        do {
            if let accountData = storedAccountNumber.data(using: .utf8) {
                try store.write(accountData, for: .lastUsedAccount)
            } else {
                logger.error("Failed to encode account number into utf-8 data.")
            }
        } catch {
            logger.error(error: error, message: "Failed to store last used account.")
        }

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
            let result = Result {
                try self.migrateSettings(
                    store: store,
                    parser: parser,
                    settings: self.legacySettings,
                    accountData: try self.accountCompletion.get(),
                    devices: try self.devicesCompletion.get()
                )
            }
            completion(result.error)
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

        let newDeviceState: DeviceState
        if let device = device {
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
            newDeviceState = DeviceState.loggedIn(
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
        } else {
            logger.debug(
                """
                Failed to find a device matching public key from legacy settings. \
                Set device state to logged out.
                """
            )

            newDeviceState = .loggedOut
        }

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
