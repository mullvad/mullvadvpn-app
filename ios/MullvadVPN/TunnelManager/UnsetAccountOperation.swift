//
//  UnsetAccountOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 16/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging

class UnsetAccountOperation: AsyncOperation {
    typealias WillDeleteVPNConfigurationHandler = () -> Void
    typealias CompletionHandler = (OperationCompletion<(), TunnelManager.Error>) -> Void

    private let queue: DispatchQueue
    private let state: TunnelManager.State
    private let restClient: REST.Client

    private var willDeleteVPNConfigurationHandler: WillDeleteVPNConfigurationHandler?
    private var completionHandler: CompletionHandler?

    private let logger = Logger(label: "TunnelManager.UnsetAccountOperation")

    init(queue: DispatchQueue, state: TunnelManager.State, restClient: REST.Client, willDeleteVPNConfigurationHandler: @escaping WillDeleteVPNConfigurationHandler, completionHandler: @escaping CompletionHandler) {
        self.queue = queue
        self.state = state
        self.restClient = restClient
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

    private func execute(_ completionHandler: @escaping (OperationCompletion<(), TunnelManager.Error>) -> Void) {
        guard !isCancelled else {
            completionHandler(.cancelled)
            return
        }

        guard let tunnelInfo = state.tunnelInfo else {
            self.logger.debug("Account is not set, so nothing to do.")

            completionHandler(.failure(.missingAccount))
            return
        }

        let accountToken = tunnelInfo.token
        let publicKey = tunnelInfo.tunnelSettings.interface.publicKey

        _ = REST.Client.shared.deleteWireguardKey(token: accountToken, publicKey: publicKey)
            .execute(retryStrategy: .default) { result in
                self.queue.async {
                    self.didDeleteWireguardKey(
                        result: result,
                        accountToken: accountToken,
                        completionHandler: completionHandler
                    )
                }
            }
    }

    private func didDeleteWireguardKey(result: Result<(), REST.Error>, accountToken: String, completionHandler: @escaping (OperationCompletion<(), TunnelManager.Error>) -> Void) {
        switch result {
        case .success:
            logger.warning("Deleted WireGuard key on server")

        case .failure(let error):
            if case .server(.pubKeyNotFound) = error {
                logger.debug("WireGuard key was not found on server")
            } else {
                logger.error(chainedError: error, message: "Failed to delete WireGuard key on server")
            }
        }

        willDeleteVPNConfigurationHandler?()
        willDeleteVPNConfigurationHandler = nil

        // Reset tunnel state to disconnected
        self.state.tunnelState = .disconnected

        // Remove tunnel info
        self.state.tunnelInfo = nil

        // Remove settings from Keychain
        if case .failure(let error) = TunnelSettingsManager.remove(searchTerm: .accountToken(accountToken)) {
            // Ignore Keychain errors because that normally means that the Keychain
            // configuration was already removed and we shouldn't be blocking the
            // user from logging out
            logger.error(
                chainedError: error,
                message: "Failure to remove tunnel setting from keychain when unsetting user account"
            )
        }

        // Finish immediately if tunnel provider is not set.
        guard let tunnelProvider = state.tunnelProvider else {
            completionHandler(.success(()))
            return
        }

        // Remove VPN configuration
        tunnelProvider.removeFromPreferences { error in
            self.queue.async {
                if let error = error {
                    // Ignore error but log it
                    self.logger.error(
                        chainedError: TunnelManager.Error.removeVPNConfiguration(error),
                        message: "Failure to remove system VPN configuration when unsetting user account."
                    )
                } else {
                    self.state.setTunnelProvider(nil, shouldRefreshTunnelState: false)
                }

                completionHandler(.success(()))
            }
        }
    }
}
