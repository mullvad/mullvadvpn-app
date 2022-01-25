//
//  LoadTunnelOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 16/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging

protocol LoadTunnelOperationDelegate: AnyObject {
    func operation(_ operation: Operation, didSetTunnelInfo newTunnelInfo: TunnelInfo)
    func operation(_ operation: Operation, didSetTunnelProvider newTunnelProvider: TunnelProviderManagerType)
    func operation(_ operation: Operation, didFinishLoadingTunnelWithCompletion completion: OperationCompletion<(), TunnelManager.Error>)
}

class LoadTunnelOperation: BaseTunnelOperation<(), TunnelManager.Error> {
    private let token: String?
    private weak var delegate: LoadTunnelOperationDelegate?

    private let logger = Logger(label: "TunnelManager.LoadTunnelOperation")

    init(queue: DispatchQueue, token: String?, delegate: LoadTunnelOperationDelegate) {
        self.token = token
        self.delegate = delegate

        super.init(queue: queue)
    }

    override func main() {
        queue.async {
            guard !self.isCancelled else {
                self.completeOperation(completion: .cancelled)
                return
            }

            // Migrate the tunnel settings if needed
            if let token = self.token {
                let migrationResult = self.migrateTunnelSettings(accountToken: token)

                if case .failure(let migrationError) = migrationResult {
                    self.completeOperation(completion: .failure(migrationError))
                    return
                }
            }

            TunnelProviderManagerType.loadAllFromPreferences { tunnels, error in
                self.queue.async {
                    if let error = error {
                        self.completeOperation(completion: .failure(.loadAllVPNConfigurations(error)))
                    } else {
                        self.didLoadTunnels(tunnels) { error in
                            self.completeOperation(completion: error.map { .failure($0) } ?? .success(()))
                        }
                    }
                }
            }
        }
    }

    private func didLoadTunnels(_ tunnels: [TunnelProviderManagerType]?, completionHandler: @escaping (TunnelManager.Error?) -> Void) {
        if let tunnelProvider = tunnels?.first {
            if let token = self.token {
                // Case 1: tunnel exists and account token is set.
                // Verify that tunnel can access the configuration via the persistent keychain reference
                // stored in `passwordReference` field of VPN configuration.
                self.handleTunnelConsistency(tunnelProvider: tunnelProvider, token: token, completionHandler: completionHandler)
            } else {
                // Case 2: tunnel exists but account token is unset.
                // Remove the orphaned tunnel.
                tunnelProvider.removeFromPreferences { error in
                    self.queue.async {
                        completionHandler(error.map { .removeInconsistentVPNConfiguration($0) })
                    }
                }
            }
        } else {
            if let token = self.token {
                // Case 3: tunnel does not exist but the account token is set.
                // Verify that tunnel settings exists in keychain.
                let tunnelSettingsResult = TunnelSettingsManager.load(searchTerm: .accountToken(token))
                   .mapError { TunnelManager.Error.readTunnelSettings($0) }

                switch tunnelSettingsResult {
                case .success(let keychainEntry):
                    let tunnelInfo = TunnelInfo(
                        token: keychainEntry.accountToken,
                        tunnelSettings: keychainEntry.tunnelSettings
                    )

                    self.delegate?.operation(self, didSetTunnelInfo: tunnelInfo)
                    completionHandler(nil)

                case .failure(let error):
                    completionHandler(error)
                }
            } else {
                // Case 4: no tunnels exist and account token is unset.
                completionHandler(nil)
            }
        }
    }

    private func handleTunnelConsistency(tunnelProvider: TunnelProviderManagerType, token: String, completionHandler: @escaping (TunnelManager.Error?) -> Void) {
        let verificationResult = verifyTunnel(tunnelProvider: tunnelProvider, expectedAccountToken: token)
        let tunnelSettingsResult = TunnelSettingsManager.load(searchTerm: .accountToken(token))
            .mapError { TunnelManager.Error.readTunnelSettings($0) }

        switch (verificationResult, tunnelSettingsResult) {
        case (.success(true), .success(let keychainEntry)):
            let tunnelInfo = TunnelInfo(token: token, tunnelSettings: keychainEntry.tunnelSettings)

            delegate?.operation(self, didSetTunnelInfo: tunnelInfo)
            delegate?.operation(self, didSetTunnelProvider: tunnelProvider)

            completionHandler(nil)

        // Remove the tunnel with corrupt configuration.
        // It will be re-created upon the first attempt to connect the tunnel.
        case (.success(false), .success(let keychainEntry)):
            tunnelProvider.removeFromPreferences { error in
                self.queue.async {
                    if let error = error {
                        completionHandler(.removeInconsistentVPNConfiguration(error))
                    } else {
                        let tunnelInfo = TunnelInfo(token: token, tunnelSettings: keychainEntry.tunnelSettings)

                        self.delegate?.operation(self, didSetTunnelInfo: tunnelInfo)
                        completionHandler(nil)
                    }
                }
            }

        // Remove the tunnel when failed to verify it but successfuly loaded the tunnel
        // settings.
        case (.failure(let verificationError), .success(let keychainEntry)):
            logger.error(chainedError: verificationError, message: "Failed to verify the tunnel but successfully loaded the tunnel settings. Removing the tunnel.")

            // Remove the tunnel with corrupt configuration.
            // It will be re-created upon the first attempt to connect the tunnel.
            tunnelProvider.removeFromPreferences { error in
                self.queue.async {
                    if let error = error {
                        completionHandler(.removeInconsistentVPNConfiguration(error))
                    } else {
                        let tunnelInfo = TunnelInfo(token: token, tunnelSettings: keychainEntry.tunnelSettings)
                        self.delegate?.operation(self, didSetTunnelInfo: tunnelInfo)

                        completionHandler(nil)
                    }
                }
            }

        // Remove the tunnel when failed to verify the tunnel and load tunnel settings.
        case (.failure(let verificationError), .failure(_)):
            logger.error(chainedError: verificationError, message: "Failed to verify the tunnel and load tunnel settings. Removing the tunnel.")

            tunnelProvider.removeFromPreferences { error in
                self.queue.async {
                    if let error = error {
                        completionHandler(.removeInconsistentVPNConfiguration(error))
                    } else {
                        completionHandler(verificationError)
                    }
                }
            }

        // Remove the tunnel when the app is not able to read tunnel settings
        case (.success(_), .failure(let settingsReadError)):
            self.logger.error(chainedError: settingsReadError, message: "Failed to load tunnel settings. Removing the tunnel.")

            tunnelProvider.removeFromPreferences { error in
                self.queue.async {
                    if let error = error {
                        completionHandler(.removeInconsistentVPNConfiguration(error))
                    } else {
                        completionHandler(settingsReadError)
                    }
                }
            }
        }
    }

    private func verifyTunnel(tunnelProvider: TunnelProviderManagerType, expectedAccountToken accountToken: String) -> Result<Bool, TunnelManager.Error> {
        // Check that the VPN configuration points to the same account token
        guard let username = tunnelProvider.protocolConfiguration?.username, username == accountToken else {
            logger.warning("The token assigned to the VPN configuration does not match the logged in account.")
            return .success(false)
        }

        // Check that the passwordReference, containing the keychain reference for tunnel
        // configuration, is set.
        guard let keychainReference = tunnelProvider.protocolConfiguration?.passwordReference else {
            logger.warning("VPN configuration is missing the passwordReference.")
            return .success(false)
        }

        // Verify that the keychain reference points to the existing entry in Keychain.
        // Bad reference is possible when migrating the user data from one device to the other.
        return TunnelSettingsManager.exists(searchTerm: .persistentReference(keychainReference))
            .mapError { (error) -> TunnelManager.Error in
                logger.error(chainedError: error, message: "Failed to verify the persistent keychain reference for tunnel settings.")

                return .readTunnelSettings(error)
            }
    }

    private func migrateTunnelSettings(accountToken: String) -> Result<Bool, TunnelManager.Error> {
        let result = TunnelSettingsManager
            .migrateKeychainEntry(searchTerm: .accountToken(accountToken))
            .mapError { (error) -> TunnelManager.Error in
                return .migrateTunnelSettings(error)
            }

        switch result {
        case .success(let migrated):
            if migrated {
                logger.info("Migrated Keychain tunnel configuration.")
            } else {
                logger.info("Tunnel settings are up to date. No migration needed.")
            }

        case .failure(let error):
            logger.error(chainedError: error)
        }

        return result
    }

    override func completeOperation(completion: OperationCompletion<(), TunnelManager.Error>) {
        delegate?.operation(self, didFinishLoadingTunnelWithCompletion: completion)
        delegate = nil

        super.completeOperation(completion: completion)
    }
}
