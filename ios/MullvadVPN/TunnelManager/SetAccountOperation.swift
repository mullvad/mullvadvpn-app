//
//  SetAccountOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 16/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import class WireGuardKitTypes.PublicKey
import Logging

class SetAccountOperation: AsyncOperation {
    typealias WillDeleteVPNConfigurationHandler = () -> Void
    typealias CompletionHandler = (OperationCompletion<(), TunnelManager.Error>) -> Void

    private let queue: DispatchQueue
    private let state: TunnelManager.State
    private let restClient: REST.Client
    private let accountToken: String?

    private var willDeleteVPNConfigurationHandler: WillDeleteVPNConfigurationHandler?
    private var completionHandler: CompletionHandler?

    private let logger = Logger(label: "TunnelManager.SetAccountOperation")

    init(queue: DispatchQueue, state: TunnelManager.State, restClient: REST.Client, accountToken: String?, willDeleteVPNConfigurationHandler: @escaping WillDeleteVPNConfigurationHandler, completionHandler: @escaping CompletionHandler) {
        self.queue = queue
        self.state = state
        self.restClient = restClient
        self.accountToken = accountToken
        self.willDeleteVPNConfigurationHandler = willDeleteVPNConfigurationHandler
        self.completionHandler = completionHandler
    }

    override func main() {
        queue.async {
            self.execute { completion in
                self.completionHandler?(completion)
                self.completionHandler = nil

                self.finish()
            }
        }
    }

    private func execute(completionHandler: @escaping CompletionHandler) {
        guard !isCancelled else {
            completionHandler(.cancelled)
            return
        }

        // Delete current account key and configuration if set.
        if let tunnelInfo = state.tunnelInfo, tunnelInfo.token != accountToken {
            let currentAccountToken = tunnelInfo.token
            let currentPublicKey = tunnelInfo.tunnelSettings.interface.publicKey
            let nextPublicKey = tunnelInfo.tunnelSettings.interface.nextPrivateKey?.publicKey

            logger.debug("Unset current account token.")

            deleteOldAccount(accountToken: currentAccountToken, currentPublicKey: currentPublicKey, nextPublicKey: nextPublicKey) {
                self.setNewAccount(completionHandler: completionHandler)
            }
        } else {
            setNewAccount(completionHandler: completionHandler)
        }
    }

    private func setNewAccount(completionHandler: @escaping CompletionHandler) {
        guard let accountToken = accountToken else {
            logger.debug("Account token is unset.")
            completionHandler(.success(()))
            return
        }

        logger.debug("Set new account token.")

        switch makeTunnelSettings(accountToken: accountToken) {
        case .success(let tunnelSettings):
            let interfaceSettings = tunnelSettings.interface

            if let newPrivateKey = interfaceSettings.nextPrivateKey {
                // Replace key if key rotation had failed.
                replaceOldAccountKey(
                    accountToken: accountToken,
                    oldPrivateKey: interfaceSettings.privateKey,
                    newPrivateKey: newPrivateKey,
                    completionHandler: completionHandler
                )
            } else if interfaceSettings.addresses.isEmpty {
                // Push key if interface addresses were not received yet
                pushNewAccountKey(
                    accountToken: accountToken,
                    publicKey: interfaceSettings.publicKey,
                    completionHandler: completionHandler
                )
            } else {
                state.tunnelInfo = TunnelInfo(
                    token: accountToken,
                    tunnelSettings: tunnelSettings
                )
                completionHandler(.success(()))
            }

        case .failure(let error):
            logger.error(chainedError: error, message: "Failed to make new account settings.")
            completionHandler(.failure(error))
        }
    }

    private func makeTunnelSettings(accountToken: String) -> Result<TunnelSettings, TunnelManager.Error> {
        return TunnelSettingsManager.load(searchTerm: .accountToken(accountToken))
            .mapError { TunnelManager.Error.readTunnelSettings($0) }
            .map { $0.tunnelSettings }
            .flatMapError { error in
                if case .readTunnelSettings(.lookupEntry(.itemNotFound)) = error {
                    let defaultConfiguration = TunnelSettings()

                    return TunnelSettingsManager
                        .add(configuration: defaultConfiguration, account: accountToken)
                        .mapError { .addTunnelSettings($0) }
                        .map { defaultConfiguration }
                } else {
                    return .failure(error)
                }
            }
    }

    private func deleteOldAccount(accountToken: String, currentPublicKey: PublicKey, nextPublicKey: PublicKey?, completionHandler: @escaping () -> Void) {
        let dispatchGroup = DispatchGroup()

        let keysToDelete = [currentPublicKey, nextPublicKey].compactMap { $0 }

        logger.debug("Deleting \(keysToDelete.count) key(s) from old account.")

        for (index, publicKey) in keysToDelete.enumerated() {
            dispatchGroup.enter()
            _ = REST.Client.shared.deleteWireguardKey(token: accountToken, publicKey: publicKey)
                .execute(retryStrategy: .default) { result in
                    self.queue.async {
                        switch result {
                        case .success:
                            self.logger.info("Removed old key (\(index)) from server.")

                        case .failure(.server(.pubKeyNotFound)):
                            self.logger.debug("Old key (\(index)) was not found on server.")

                        case .failure(let error):
                            self.logger.error(chainedError: error, message: "Failed to delete old key (\(index)) on server.")
                        }

                        dispatchGroup.leave()
                    }
                }
        }

        dispatchGroup.notify(queue: queue) {
            self.deleteKeychainEntryAndVPNConfiguration(accountToken: accountToken, completionHandler: completionHandler)
        }
    }

    private func deleteKeychainEntryAndVPNConfiguration(accountToken: String, completionHandler: @escaping () -> Void) {
        // Tell the caller to unsubscribe from VPN status notifications.
        willDeleteVPNConfigurationHandler?()
        willDeleteVPNConfigurationHandler = nil

        // Reset tunnel state to disconnected
        state.tunnelState = .disconnected

        // Remove tunnel info
        state.tunnelInfo = nil

        // Remove settings from Keychain
        if case .failure(let error) = TunnelSettingsManager.remove(searchTerm: .accountToken(accountToken)) {
            // Ignore Keychain errors because that normally means that the Keychain
            // configuration was already removed and we shouldn't be blocking the
            // user from logging out
            logger.error(
                chainedError: error,
                message: "Failed to delete old account settings."
            )
        }

        // Finish immediately if tunnel provider is not set.
        guard let tunnel = state.tunnel else {
            completionHandler()
            return
        }

        // Remove VPN configuration
        tunnel.removeFromPreferences { error in
            self.queue.async {
                // Ignore error but log it
                if let error = error {
                    self.logger.error(
                        chainedError: AnyChainedError(error),
                        message: "Failed to remove VPN configuration."
                    )
                }

                self.state.setTunnel(nil, shouldRefreshTunnelState: false)

                completionHandler()
            }
        }
    }

    private func replaceOldAccountKey(accountToken: String, oldPrivateKey: PrivateKeyWithMetadata, newPrivateKey: PrivateKeyWithMetadata, completionHandler: @escaping CompletionHandler) {
        _ = restClient.replaceWireguardKey(token: accountToken, oldPublicKey: oldPrivateKey.publicKey, newPublicKey: newPrivateKey.publicKey)
            .execute(retryStrategy: .default) { result in
                self.queue.async {
                    switch result {
                    case .success(let associatedAddresses):
                        self.logger.debug("Replaced old key with new key on server.")

                        self.saveAssociatedAddresses(associatedAddresses, accountToken: accountToken, newPrivateKey: newPrivateKey, completionHandler: completionHandler)

                    case .failure(let error):
                        self.logger.error(chainedError: error, message: "Failed to replace old key with new key on server.")

                        completionHandler(.failure(.replaceWireguardKey(error)))
                    }
                }
            }
    }

    private func pushNewAccountKey(accountToken: String, publicKey: PublicKey, completionHandler: @escaping CompletionHandler) {
        _ = restClient.pushWireguardKey(token: accountToken, publicKey: publicKey)
            .execute(retryStrategy: .default) { result in
                self.queue.async {
                    switch result {
                    case .success(let associatedAddresses):
                        self.logger.debug("Pushed new key to server.")

                        self.saveAssociatedAddresses(associatedAddresses, accountToken: accountToken, newPrivateKey: nil, completionHandler: completionHandler)

                    case .failure(let error):
                        self.logger.error(chainedError: error, message: "Failed to push new key to server.")

                        completionHandler(.failure(.pushWireguardKey(error)))
                    }
                }
            }
    }

    private func saveAssociatedAddresses(_ associatedAddresses: REST.WireguardAddressesResponse, accountToken: String, newPrivateKey: PrivateKeyWithMetadata?, completionHandler: @escaping (OperationCompletion<(), TunnelManager.Error>) -> Void) {
        let saveResult = TunnelSettingsManager.update(searchTerm: .accountToken(accountToken)) { tunnelSettings in
            tunnelSettings.interface.addresses = [
                associatedAddresses.ipv4Address,
                associatedAddresses.ipv6Address
            ]

            if let newPrivateKey = newPrivateKey {
                tunnelSettings.interface.privateKey = newPrivateKey
                tunnelSettings.interface.nextPrivateKey = nil
            }
        }

        switch saveResult {
        case .success(let newTunnelSettings):
            logger.debug("Saved associated addresses.")

            state.tunnelInfo = TunnelInfo(
                token: accountToken,
                tunnelSettings: newTunnelSettings
            )

            completionHandler(.success(()))

        case .failure(let error):
            logger.error(chainedError: error, message: "Failed to save associated addresses.")

            completionHandler(.failure(.updateTunnelSettings(error)))
        }
    }
}
