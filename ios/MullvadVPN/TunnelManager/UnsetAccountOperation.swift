//
//  UnsetAccountOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 16/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging

protocol UnsetAccountOperationDelegate: AnyObject {
    func operationDidRequestTunnelInfo(_ operation: Operation) -> TunnelInfo?
    func operationDidRequestTunnelProvider(_ operation: Operation) -> TunnelProviderManagerType?
    func operationWillDeleteVPNConfiguration(_ operation: Operation)
    func operationDidUnsetAccount(_ operation: Operation)
}

class UnsetAccountOperation: AsyncOperation {
    private let queue: DispatchQueue
    private let restClient: REST.Client
    private weak var delegate: UnsetAccountOperationDelegate?

    private let logger = Logger(label: "TunnelManager.UnsetAccountOperation")

    init(queue: DispatchQueue, restClient: REST.Client, delegate: UnsetAccountOperationDelegate) {
        self.queue = queue
        self.restClient = restClient
        self.delegate = delegate
    }

    override func main() {
        queue.async {
            guard !self.isCancelled else {
                self.finish()
                return
            }

            guard let tunnelInfo = self.delegate?.operationDidRequestTunnelInfo(self) else {
                self.logger.debug("Account is not set, so nothing to do.")
                self.finish()
                return
            }

            let token = tunnelInfo.token
            let publicKey = tunnelInfo.tunnelSettings.interface.publicKey

            _ = REST.Client.shared.deleteWireguardKey(token: token, publicKey: publicKey)
                .execute(retryStrategy: .default) { result in
                    self.queue.async {
                        switch result {
                        case .success:
                            self.logger.warning("Deleted WireGuard key on server")

                        case .failure(let error):
                            if case .server(.pubKeyNotFound) = error {
                                self.logger.debug("WireGuard key was not found on server")
                            } else {
                                self.logger.error(chainedError: error, message: "Failed to delete WireGuard key on server")
                            }
                        }

                        self.delegate?.operationWillDeleteVPNConfiguration(self)

                        // Remove settings from Keychain
                        if case .failure(let error) = TunnelSettingsManager.remove(searchTerm: .accountToken(tunnelInfo.token)) {
                            // Ignore Keychain errors because that normally means that the Keychain
                            // configuration was already removed and we shouldn't be blocking the
                            // user from logging out
                            self.logger.error(
                                chainedError: error,
                                message: "Failure to remove tunnel setting from keychain when unsetting user account"
                            )
                        }

                        // Finish immediately if tunnel provider is not set.
                        guard let tunnelProvider = self.delegate?.operationDidRequestTunnelProvider(self) else {
                            self.delegate?.operationDidUnsetAccount(self)
                            self.finish()
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
                                }

                                self.delegate?.operationDidUnsetAccount(self)
                                self.finish()
                            }
                        }
                    }
                }
        }
    }
}
