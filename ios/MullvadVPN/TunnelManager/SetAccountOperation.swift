//
//  SetAccountOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 16/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import class WireGuardKitTypes.PublicKey
import class WireGuardKitTypes.PrivateKey
import Logging

enum SetAccountAction {
    /// Set new account.
    case new

    /// Set existing account.
    case existing(String)

    /// Unset account.
    case unset

    var taskName: String {
        switch self {
        case .new:
            return "Set new account"
        case .existing:
            return "Set existing account"
        case .unset:
            return "Unset account"
        }
    }
}

class SetAccountOperation: ResultOperation<StoredAccountData?, TunnelManager.Error> {
    typealias WillDeleteVPNConfigurationHandler = () -> Void

    private let state: TunnelManager.State
    private let accountsProxy: REST.AccountsProxy
    private let devicesProxy: REST.DevicesProxy
    private let action: SetAccountAction
    private var task: Cancellable?

    private var willDeleteVPNConfigurationHandler: WillDeleteVPNConfigurationHandler?
    private let logger = Logger(label: "TunnelManager.SetAccountOperation")

    init(
        dispatchQueue: DispatchQueue,
        state: TunnelManager.State,
        accountsProxy: REST.AccountsProxy,
        devicesProxy: REST.DevicesProxy,
        action: SetAccountAction,
        willDeleteVPNConfigurationHandler: @escaping WillDeleteVPNConfigurationHandler,
        completionHandler: @escaping CompletionHandler
    )
    {
        self.state = state
        self.accountsProxy = accountsProxy
        self.devicesProxy = devicesProxy
        self.action = action
        self.willDeleteVPNConfigurationHandler = willDeleteVPNConfigurationHandler

        super.init(
            dispatchQueue: dispatchQueue,
            completionQueue: dispatchQueue,
            completionHandler: completionHandler
        )
    }

    override func main() {
        if let tunnelSettings = self.state.tunnelSettings {
            self.deleteDevice(
                accountNumber: tunnelSettings.account.number,
                deviceIdentifier: tunnelSettings.device.identifier
            )
        } else {
            self.fetchAccountData()
        }
    }

    private func deleteDevice(accountNumber: String, deviceIdentifier: String) {
        logger.debug("Delete current device...")

        task = devicesProxy.deleteDevice(
            accountNumber: accountNumber,
            identifier: deviceIdentifier,
            retryStrategy: .default,
            completion: { [weak self] completion in
                self?.dispatchQueue.async {
                    self?.didDeleteDevice(completion)
                }
            })
    }

    private func didDeleteDevice(_ completion: OperationCompletion<Bool, REST.Error>) {
        let mappedCompletion = completion.mapError { error -> TunnelManager.Error in
            logger.error(chainedError: error, message: "Failed to delete device.")

            return .deleteDevice(error)
        }

        guard let isDeleted = mappedCompletion.value else {
            finish(completion: mappedCompletion.assertNoSuccess())
            return
        }

        if isDeleted {
            logger.debug("Deleted device.")
        } else {
            logger.debug("Device is already deleted.")
        }

        state.tunnelSettings = nil

        deleteSettingsAndVPNConfiguration { error in
            if let error = error {
                self.finish(completion: .failure(error))
            } else {
                self.fetchAccountData()
            }
        }
    }

    private func fetchAccountData() {
        switch action {
        case .unset:
            logger.debug("Account number is unset.")

            finish(completion: .success(nil))

        case .new:
            logger.debug("Create new account...")

            task = accountsProxy.createAccount(retryStrategy: .default) { [weak self] completion in
                self?.dispatchQueue.async {
                    self?.didCreateAccount(completion: completion)
                }
            }

        case .existing(let accountNumber):
            logger.debug("Request account data...")

            task = accountsProxy.getAccountData(
                accountNumber: accountNumber,
                retryStrategy: .default,
                completion: { [weak self] completion in
                    self?.dispatchQueue.async {
                        self?.didReceiveAccountData(accountNumber: accountNumber, completion: completion)
                    }
                })
        }
    }

    private func didCreateAccount(completion: OperationCompletion<REST.NewAccountData, REST.Error>) {
        let mappedCompletion = completion.mapError { error -> TunnelManager.Error in
            logger.error(
                chainedError: AnyChainedError(error),
                message: "Failed to create new account."
            )

            return .createAccount(error)
        }

        guard let newAccountData = mappedCompletion.value else {
            finish(completion: mappedCompletion.assertNoSuccess())
            return
        }

        logger.debug("Created new account. Updating settings...")

        createDevice(
            storedAccountData: StoredAccountData(
                identifier: newAccountData.id,
                number: newAccountData.number,
                expiry: newAccountData.expiry
            )
        )
    }

    private func didReceiveAccountData(accountNumber: String, completion: OperationCompletion<REST.AccountData, REST.Error>) {
        let mappedCompletion = completion.mapError { error -> TunnelManager.Error in
            logger.error(
                chainedError: AnyChainedError(error),
                message: "Failed to receive account data."
            )

            return .getAccountData(error)
        }

        guard let accountData = mappedCompletion.value else {
            finish(completion: mappedCompletion.assertNoSuccess())
            return
        }

        logger.debug("Received account data.")

        createDevice(
            storedAccountData: StoredAccountData(
                identifier: accountData.id,
                number: accountNumber,
                expiry: accountData.expiry
            )
        )
    }

    private func createDevice(storedAccountData: StoredAccountData) {
        logger.debug("Store last used account.")

        do {
            try SettingsManager.setLastUsedAccount(storedAccountData.number)
        } catch {
            logger.error(
                chainedError: AnyChainedError(error),
                message: "Failed to store last used account number."
            )
        }

        logger.debug("Create device...")

        let privateKey = PrivateKey()

        let request = REST.CreateDeviceRequest(
            publicKey: privateKey.publicKey,
            hijackDNS: false
        )

        task = devicesProxy.createDevice(
            accountNumber: storedAccountData.number,
            request: request,
            retryStrategy: .default,
            completion: { [weak self] completion in
                self?.dispatchQueue.async {
                    self?.didCreateDevice(
                        storedAccountData: storedAccountData,
                        privateKey: privateKey,
                        completion: completion
                    )
                }
            })
    }

    private func didCreateDevice(
        storedAccountData: StoredAccountData,
        privateKey: PrivateKey,
        completion: OperationCompletion<REST.Device, REST.Error>
    )
    {
        let mappedCompletion = completion.mapError { error -> TunnelManager.Error in
            logger.error(chainedError: error, message: "Failed to create device.")
            return .createDevice(error)
        }

        guard let device = mappedCompletion.value else {
            finish(completion: mappedCompletion.assertNoSuccess())
            return
        }

        logger.debug("Created device. Saving settings...")

        let tunnelSettings = TunnelSettingsV2(
            account: storedAccountData,
            device: StoredDeviceData(
                creationDate: device.created,
                identifier: device.id,
                name: device.name,
                hijackDNS: device.hijackDNS,
                ipv4Address: device.ipv4Address,
                ipv6Address: device.ipv6Address,
                wgKeyData: StoredWgKeyData(
                    creationDate: Date(),
                    privateKey: privateKey
                )
            ),
            relayConstraints: RelayConstraints(),
            dnsSettings: DNSSettings()
        )

        do {
            try SettingsManager.writeSettings(tunnelSettings)

            state.tunnelSettings = tunnelSettings

            finish(completion: .success(storedAccountData))
        } catch {
            logger.error(
                chainedError: AnyChainedError(error),
                message: "Failed to write settings."
            )
            finish(completion: .failure(.writeSettings(error)))
        }
    }

    private func deleteSettingsAndVPNConfiguration(
        completionHandler: @escaping (TunnelManager.Error?) -> Void
    ) {
        // Delete keychain entry.
        do {
            try SettingsManager.deleteSettings()
        } catch .itemNotFound as KeychainError {
            logger.debug("Settings are already deleted.")
        } catch {
            logger.error(
                chainedError: AnyChainedError(error),
                message: "Failed to delete settings."
            )
            completionHandler(.deleteSettings(error))
            return
        }

        // Tell the caller to unsubscribe from VPN status notifications.
        willDeleteVPNConfigurationHandler?()
        willDeleteVPNConfigurationHandler = nil

        // Reset tunnel state to disconnected
        state.tunnelStatus.reset(to: .disconnected)

        // Remove tunnel settins
        state.tunnelSettings = nil

        // Finish immediately if tunnel provider is not set.
        guard let tunnel = state.tunnel else {
            completionHandler(nil)
            return
        }

        // Remove VPN configuration
        tunnel.removeFromPreferences { error in
            self.dispatchQueue.async {
                // Ignore error but log it
                if let error = error {
                    self.logger.error(
                        chainedError: AnyChainedError(error),
                        message: "Failed to remove VPN configuration."
                    )
                }

                self.state.setTunnel(nil, shouldRefreshTunnelState: false)

                completionHandler(nil)
            }
        }
    }
}
