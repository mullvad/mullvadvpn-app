//
//  LoadTunnelOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 16/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging

class LoadTunnelOperation: AsyncOperation {
    typealias CompletionHandler = (OperationCompletion<(), TunnelManager.Error>) -> Void

    private let queue: DispatchQueue
    private let accountToken: String?
    private let state: TunnelManager.State
    private var completionHandler: CompletionHandler?

    private let logger = Logger(label: "TunnelManager.LoadTunnelOperation")

    init(queue: DispatchQueue, state: TunnelManager.State, accountToken: String?, completionHandler: @escaping CompletionHandler) {
        self.queue = queue
        self.state = state
        self.accountToken = accountToken
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

        // Migrate the tunnel settings if needed
        if let accountToken = accountToken {
            let migrationResult = migrateTunnelSettings(accountToken: accountToken)

            if case .failure(let migrationError) = migrationResult {
                completionHandler(.failure(migrationError))
                return
            }
        }

        TunnelProviderManagerType.loadAllFromPreferences { tunnels, error in
            self.queue.async {
                if let error = error {
                    completionHandler(.failure(.loadAllVPNConfigurations(error)))
                } else {
                    self.didLoadVPNConfigurations(tunnels: tunnels, completionHandler: completionHandler)
                }
            }
        }
    }

    private func didLoadVPNConfigurations(tunnels: [TunnelProviderManagerType]?, completionHandler: @escaping CompletionHandler) {
        if let tunnelProvider = tunnels?.first {
            if let accountToken = accountToken {
                // Case 1: tunnel exists and account token is set.
                // Verify that tunnel can access the configuration via the persistent keychain reference
                // stored in `passwordReference` field of VPN configuration.
                handleTunnelConsistency(tunnelProvider: tunnelProvider, accountToken: accountToken, completionHandler: completionHandler)
            } else {
                // Case 2: tunnel exists but account token is unset.
                // Remove the orphaned tunnel.
                tunnelProvider.removeFromPreferences { error in
                    self.queue.async {
                        if let error = error {
                            completionHandler(.failure(.removeInconsistentVPNConfiguration(error)))
                        } else {
                            completionHandler(.success(()))
                        }
                    }
                }
            }
        } else {
            if let accountToken = accountToken {
                // Case 3: tunnel does not exist but the account token is set.
                // Verify that tunnel settings exists in keychain.
                let tunnelSettingsResult = TunnelSettingsManager.load(searchTerm: .accountToken(accountToken))
                   .mapError { TunnelManager.Error.readTunnelSettings($0) }

                if case .success(let keychainEntry) = tunnelSettingsResult {
                    let tunnelInfo = TunnelInfo(
                        token: keychainEntry.accountToken,
                        tunnelSettings: keychainEntry.tunnelSettings
                    )

                    state.tunnelInfo = tunnelInfo
                }

                completionHandler(OperationCompletion(result: tunnelSettingsResult.map { _ in () }))
            } else {
                // Case 4: no tunnels exist and account token is unset.
                completionHandler(.success(()))
            }
        }
    }

    private func handleTunnelConsistency(tunnelProvider: TunnelProviderManagerType, accountToken: String, completionHandler: @escaping CompletionHandler) {
        let verificationResult = verifyTunnel(tunnelProvider: tunnelProvider, expectedAccountToken: accountToken)
        let tunnelSettingsResult = TunnelSettingsManager.load(searchTerm: .accountToken(accountToken))
            .mapError { TunnelManager.Error.readTunnelSettings($0) }

        switch (verificationResult, tunnelSettingsResult) {
        case (.success(true), .success(let keychainEntry)):
            let tunnelInfo = TunnelInfo(token: accountToken, tunnelSettings: keychainEntry.tunnelSettings)

            state.tunnelInfo = tunnelInfo
            state.setTunnel(Tunnel(tunnelProvider: tunnelProvider), shouldRefreshTunnelState: true)

            completionHandler(.success(()))

        // Remove the tunnel with corrupt configuration.
        // It will be re-created upon the first attempt to connect the tunnel.
        case (.success(false), .success(let keychainEntry)):
            tunnelProvider.removeFromPreferences { error in
                self.queue.async {
                    if let error = error {
                        completionHandler(.failure(.removeInconsistentVPNConfiguration(error)))
                    } else {
                        let tunnelInfo = TunnelInfo(token: accountToken, tunnelSettings: keychainEntry.tunnelSettings)
                        self.state.tunnelInfo = tunnelInfo

                        completionHandler(.success(()))
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
                        completionHandler(.failure(.removeInconsistentVPNConfiguration(error)))
                    } else {
                        let tunnelInfo = TunnelInfo(token: accountToken, tunnelSettings: keychainEntry.tunnelSettings)
                        self.state.tunnelInfo = tunnelInfo

                        completionHandler(.success(()))
                    }
                }
            }

        // Remove the tunnel when failed to verify the tunnel and load tunnel settings.
        case (.failure(let verificationError), .failure(_)):
            logger.error(chainedError: verificationError, message: "Failed to verify the tunnel and load tunnel settings. Removing the tunnel.")

            tunnelProvider.removeFromPreferences { error in
                self.queue.async {
                    if let error = error {
                        completionHandler(.failure(.removeInconsistentVPNConfiguration(error)))
                    } else {
                        completionHandler(.failure(verificationError))
                    }
                }
            }

        // Remove the tunnel when the app is not able to read tunnel settings
        case (.success(_), .failure(let settingsReadError)):
            logger.error(chainedError: settingsReadError, message: "Failed to load tunnel settings. Removing the tunnel.")

            tunnelProvider.removeFromPreferences { error in
                self.queue.async {
                    if let error = error {
                        completionHandler(.failure(.removeInconsistentVPNConfiguration(error)))
                    } else {
                        completionHandler(.failure(settingsReadError))
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
}
