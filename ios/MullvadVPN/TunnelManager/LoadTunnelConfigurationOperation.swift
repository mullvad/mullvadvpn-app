//
//  LoadTunnelConfigurationOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 16/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging

class LoadTunnelConfigurationOperation: ResultOperation<(), TunnelManager.Error> {
    private let logger = Logger(label: "LoadTunnelConfigurationOperation")
    private let state: TunnelManager.State

    init(
        dispatchQueue: DispatchQueue,
        state: TunnelManager.State,
        completionHandler: @escaping CompletionHandler
    ) {
        self.state = state

        super.init(
            dispatchQueue: dispatchQueue,
            completionQueue: dispatchQueue,
            completionHandler: completionHandler
        )
    }

    override func main() {
        TunnelProviderManagerType.loadAllFromPreferences { tunnels, error in
            self.dispatchQueue.async {
                if let error = error {
                    self.finish(completion: .failure(.loadAllVPNConfigurations(error)))
                } else {
                    self.didLoadVPNConfigurations(tunnels: tunnels)
                }
            }
        }
    }

    private func didLoadVPNConfigurations(tunnels: [TunnelProviderManagerType]?) {
        let tunnelProvider = tunnels?.first

        do {
            let tunnelSettings = try SettingsManager.readSettings()
            let tunnel = tunnelProvider.map { tunnelProvider in
                return Tunnel(tunnelProvider: tunnelProvider)
            }

            state.tunnelSettings = tunnelSettings
            state.setTunnel(tunnel, shouldRefreshTunnelState: true)

            finish(completion: .success(()))
        } catch .itemNotFound as KeychainError {
            logger.debug("Settings not found in keychain.")

            state.tunnelSettings = nil
            state.setTunnel(nil, shouldRefreshTunnelState: true)

            if let tunnelProvider = tunnelProvider {
                removeOrphanedTunnel(tunnelProvider: tunnelProvider) { error in
                    self.finish(completion: error.map { .failure($0) } ?? .success(()))
                }
            } else {
                finish(completion: .success(()))
            }
        } catch let error as DecodingError {
            state.tunnelSettings = nil
            state.setTunnel(nil, shouldRefreshTunnelState: true)

            do {
                logger.error(
                    chainedError: AnyChainedError(error),
                    message: "Cannot decode settings. Will attempt to delete them from keychain."
                )

                try SettingsManager.deleteSettings()
            } catch {
                logger.error(
                    chainedError: AnyChainedError(error),
                    message: "Failed to delete settings from keychain."
                )
            }

            let returnError: TunnelManager.Error = .readSettings(error)

            if let tunnelProvider = tunnelProvider {
                removeOrphanedTunnel(tunnelProvider: tunnelProvider) { _ in
                    self.finish(completion: .failure(returnError))
                }
            } else {
                finish(completion: .failure(returnError))
            }
        } catch {
            state.tunnelSettings = nil
            state.setTunnel(nil, shouldRefreshTunnelState: true)

            let returnError: TunnelManager.Error = .readSettings(error)

            if let tunnelProvider = tunnelProvider {
                removeOrphanedTunnel(tunnelProvider: tunnelProvider) { _ in
                    self.finish(completion: .failure(returnError))
                }
            } else {
                finish(completion: .failure(returnError))
            }
        }
    }

    private func removeOrphanedTunnel(tunnelProvider: TunnelProviderManagerType, completion: @escaping (TunnelManager.Error?) -> Void) {
        logger.debug("Remove orphaned VPN configuration.")

        tunnelProvider.removeFromPreferences { error in
            self.dispatchQueue.async {
                if let error = error {
                    self.logger.error(
                        chainedError: AnyChainedError(error),
                        message: "Failed to remove VPN configuration."
                    )
                    completion(.removeVPNConfiguration(error))
                } else {
                    completion(nil)
                }
            }
        }
    }
}
